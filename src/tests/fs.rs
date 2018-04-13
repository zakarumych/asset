use std::io::{Error, Read};

use asset::{Asset, AssetLoader};

const LICENSE_MIT: &'static str = "Copyright (c) 2016 The Asset Project Developers

Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the \"Software\"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
";

pub struct StringLoader;
impl AssetLoader<String, ()> for StringLoader {
    type Error = Error;

    fn load<R>(&mut self, _: (), mut reader: R) -> Result<String, Error>
    where
        R: Read,
    {
        let mut text = String::new();
        reader.read_to_string(&mut text)?;
        Ok(text)
    }
}

impl Asset for String {
    const KIND: &'static str = "String";

    type Loader = StringLoader;
}

#[test]
fn fs() {
    use store::fs::FsStore;
    use store::Store;
    let mut fs = FsStore::new();
    fs.add(env!("CARGO_MANIFEST_DIR"));
    assert_eq!(String::from(LICENSE_MIT), fs.fetch("LICENSE-MIT".as_ref()).and_then(|r| StringLoader.load((), r)).unwrap());
}

