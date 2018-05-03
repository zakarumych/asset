
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;

use failure::Error;

use asset::{Asset, AssetLoader};
use store::Store;

trait AnyStore<I> {
    fn fetch(&mut self, id: &I) -> Result<&mut Read, Error>;
    fn close(&mut self);
}

impl<I, S> AnyStore<I> for (S, Option<S::Reader>)
where
    S: Store<I>,
    S::Error: Into<Error>,
{
    fn fetch(&mut self, id: &I) -> Result<&mut Read, Error> {
        let reader = self.0.fetch(id).map_err(|e| e.into().context(format!("Failed to fetch asset from <{}> store", S::KIND)))?;
        self.1 = Some(reader);
        Ok(self.1.as_mut().unwrap())
    }

    fn close(&mut self) {
        self.1 = None;
    }
}

/// Manages loaders and caches assets.
/// Should be able to load any asset type.
pub struct Assets<I = PathBuf> {
    stores: Vec<Box<AnyStore<I> + Send + Sync>>,
    loaders: HashMap<TypeId, Box<Any + Send + Sync>>,
    cache: HashMap<(I, TypeId), Box<Any + Send + Sync>>,
}

impl<I> Default for Assets<I>
where
    I: Hash + Eq,
{
    fn default() -> Self {
        Assets {
            stores: Default::default(),
            loaders: Default::default(),
            cache: Default::default(),
        }
    }
}

impl<I> Assets<I>
where
    I: Debug + Hash + Eq,
{
    /// Create new `Assets`
    pub fn new() -> Self {
        Self::default()
    }

    /// Add store to the manager.
    pub fn add_store<S>(&mut self, store: S)
    where
        S: Store<I> + Send + Sync + 'static,
        S::Error: Into<Error>,
        S::Reader: Send + Sync + 'static,
    {
        self.stores.push(Box::new((store, None)));
    }

    /// Add store to the manager.
    pub fn with_store<S>(mut self, store: S) -> Self
    where
        S: Store<I> + Send + Sync + 'static,
        S::Error: Into<Error>,
        S::Reader: Send + Sync + 'static,
    {
        self.add_store(store);
        self
    }

    /// Register asset loader.
    pub fn add_loader<L>(&mut self, loader: L)
    where
        L: Any + Send + Sync,
    {
        self.loaders.insert(TypeId::of::<L>(), Box::new(loader));
    }

    /// Register asset loader.
    pub fn with_loader<L>(mut self, loader: L) -> Self
    where
        L: Any + Send + Sync,
    {
        self.add_loader(loader);
        self
    }

    /// Load asset from managed store.
    /// Or get cached asset.
    pub fn load_with<A, F>(&mut self, id: I, format: F, loader: &mut A::Loader) -> Result<Arc<A>, Error>
    where
        A: Asset + 'static,
        A::Loader: AssetLoader<A, F>,
        <A::Loader as AssetLoader<A, F>>::Error: Into<Error>,
    {
        use std::collections::hash_map::Entry;
        use failure::err_msg;

        let id = id.into();

        match self.cache.entry((id, TypeId::of::<A>())) {
            Entry::Vacant(vacant) => {
                let mut errors = Vec::new();
                for store in &mut self.stores {
                    match store.fetch(&vacant.key().0) {
                        Ok(reader) => {
                            let asset = loader.load(format, reader).map_err(|e| e.into().context(format!("Failed to load asset <{}>", A::KIND)))?;
                            let asset = Arc::new(asset);
                            vacant.insert(Box::new(asset.clone()));
                            return Ok(asset);
                        }
                        Err(err) => {
                            errors.push(err);
                        }
                    }
                }

                Err(errors.into_iter().fold(err_msg(format!("Failed to find asset <{}>", A::KIND)), |a, e| {
                    e.context(a).into()
                }))
            }
            Entry::Occupied(occupied) => {
                let asset: &Arc<A> = Any::downcast_ref::<Arc<A>>(&**occupied.get()).expect("Cached assets are mapped by `TypeId`");
                Ok(Arc::clone(asset))
            }
        }
    }

    /// Load asset from managed store.
    /// Or get cached asset.
    pub fn load<A, F>(&mut self, id: I, format: F) -> Result<Arc<A>, Error>
    where
        A: Asset + 'static,
        A::Loader: AssetLoader<A, F>,
        <A::Loader as AssetLoader<A, F>>::Error: Into<Error>,
    {
        use std::collections::hash_map::Entry;
        use failure::err_msg;

        let id = id.into();

        match self.cache.entry((id, TypeId::of::<A>())) {
            Entry::Vacant(vacant) => {
                let loader = self.loaders.get_mut(&TypeId::of::<A::Loader>())
                    .ok_or_else(|| err_msg(format!("Loader for <{}> is not registered", A::KIND)))?;
                let loader = Any::downcast_mut::<A::Loader>(&mut **loader).expect("Loaders are mapped by `TypeId`");

                let mut errors = Vec::new();
                for store in &mut self.stores {
                    match store.fetch(&vacant.key().0) {
                        Ok(reader) => {
                            let asset: A = loader.load(format, reader).map_err(|e| e.into().context(format!("Failed to load asset <{}>", A::KIND)))?;
                            let asset = Arc::new(asset);
                            vacant.insert(Box::new(Arc::clone(&asset)));
                            return Ok(asset);
                        }
                        Err(err) => {
                            errors.push(err);
                        }
                    }
                }

                Err(errors.into_iter().fold(err_msg(format!("Failed to find asset <{}>", A::KIND)), |a, e| {
                    e.context(a).into()
                }))
            }
            Entry::Occupied(occupied) => {
                let asset: &Arc<A> = Any::downcast_ref::<Arc<A>>(&**occupied.get()).expect("Cached assets are mapped by `TypeId`");
                Ok(Arc::clone(asset))
            }
        }
    }
}
