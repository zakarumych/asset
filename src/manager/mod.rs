
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::io::Read;
use std::sync::Arc;

use failure::{Error, Fail};

use asset::{Asset, AssetLoader};
use store::Store;

trait AnyStore<I> {
    fn fetch(&mut self, id: &I) -> Result<&mut Read, Error>;
    fn close(&mut self);
}

impl<I, S> AnyStore<I> for (S, Option<S::Reader>)
where
    S: Store<I>,
    S::Error: Fail,
{
    fn fetch(&mut self, id: &I) -> Result<&mut Read, Error> {
        use failure::ResultExt;
        let reader = self.0.fetch(id).with_context(|_| format!("Failed to fetch asset from <{}> store", S::KIND))?;
        self.1 = Some(reader);
        Ok(self.1.as_mut().unwrap())
    }

    fn close(&mut self) {
        self.1 = None;
    }
}


/// Manages loaders and caches assets.
/// Should be able to load any asset type.
pub struct AssetManager<I> {
    stores: Vec<Box<AnyStore<I>>>,
    loaders: HashMap<TypeId, Box<Any>>,
    cache: HashMap<(I, TypeId), Box<Any>>,
}

impl<I> AssetManager<I>
where
    I: Debug + Hash + Eq,
{
    /// Add store to the manager.
    pub fn add_store<S>(&mut self, store: S)
    where
        S: Store<I> + 'static,
        S::Error: Fail,
        S::Reader: 'static,
    {
        self.stores.push(Box::new((store, None)));
    }

    /// Register asset loader.
    pub fn register<L>(&mut self, loader: L)
    where
        L: Any,
    {
        self.loaders.insert(TypeId::of::<L>(), Box::new(loader));
    }

    /// Load asset from managed store.
    /// Or get cached asset.
    pub fn load<A, F>(&mut self, id: I, format: F) -> Result<Arc<A>, Error>
    where
        A: Asset + 'static,
        A::Loader: AssetLoader<A, F>,
        <A::Loader as AssetLoader<A, F>>::Error: Fail,
    {
        use std::collections::hash_map::Entry;
        use failure::{err_msg, ResultExt};

        match self.cache.entry((id, TypeId::of::<A>())) {
            Entry::Vacant(vacant) => {
                let loader = self.loaders.get_mut(&TypeId::of::<A::Loader>())
                    .ok_or_else(|| err_msg(format!("Loader for <{}> is not registered", A::KIND)))?;
                let loader = loader.downcast_mut::<A::Loader>().expect("Loaders are mapped by `TypeId`");

                let mut errors = Vec::new();
                for store in &mut self.stores {
                    match store.fetch(&vacant.key().0) {
                        Ok(reader) => {
                            let asset = loader.load(format, reader).with_context(|_| format!("Failed to load asset <{}>", A::KIND))?;
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
                let asset: &Arc<A> = occupied.get().downcast_ref::<Arc<A>>().expect("Cached assets are mapped by `TypeId`");
                Ok(Arc::clone(asset))
            }
        }
    }
}
