
#[cfg(feature="fs")]
mod fs;
#[cfg(feature="fs")]
mod manager;

use std::io::Read;

use asset::{Asset, AssetLoader};
use ron;

#[derive(Debug, PartialEq, Eq, Deserialize)]
struct Foo {
    foo: u32,
}

impl Asset for Foo {
    const KIND: &'static str = "Foo";
    type Loader = FooLoader;
}

struct FooLoader;

impl AssetLoader<Foo, ()> for FooLoader {

    type Error = ron::de::Error;

    fn load<R>(&mut self, _: (), reader: R) -> Result<Foo, ron::de::Error>
    where
        R: Read,
    {
        ron::de::from_reader(reader)
    }
}


#[test]
fn foo() {
    assert_eq!(FooLoader.load((), &b"Foo(foo: 42)"[..]), Ok(Foo { foo: 42, }));
}

