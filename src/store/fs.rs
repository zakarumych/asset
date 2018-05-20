

use std::io;
use std::fs::File;
use std::path::{Path, PathBuf};

use store::Store;

/// `FsStore` fetches data from files in local filesystem.
#[derive(Debug)]
pub struct FsStore {
    roots: Vec<PathBuf>,
    ignore_ext: bool,
}

impl FsStore {
    /// Create new `FsStore`.
    pub fn new() -> Self {
        FsStore {
            roots: Vec::new(),
            ignore_ext: false,
        }
    }

    /// Add new search directory.
    pub fn add_path<P>(&mut self, path: P)
    where
        P: Into<PathBuf>,
    {
        self.roots.push(path.into());
    }

    /// Add new search directory.
    pub fn with_path<P>(mut self, path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        self.add_path(path);
        self
    }

    /// Set if store should ignore extensions of files.
    pub fn set_ignore_ext(&mut self, ignore: bool) {
        self.ignore_ext = ignore;
    }

    /// Set if store should ignore extensions of files.
    pub fn with_ignore_ext(mut self, ignore: bool) -> Self {
        self.set_ignore_ext(ignore);
        self
    }

    /// Find file by name.
    pub fn find<P>(&self, path: P) -> Result<File, io::Error>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let stem = path.file_stem().expect("Target must be file, not directory");

        for root in &self.roots {
            let mut path = root.join(path);
            if self.ignore_ext {
                path.set_file_name(stem);
            }
            match File::open(path) {
                Ok(file) => return Ok(file),
                Err(err) => {
                    if err.kind() != io::ErrorKind::NotFound {
                        return Err(err)
                    }
                }
            }
        }

        Err(io::ErrorKind::NotFound.into())
    }
}

impl<P> Store<P> for FsStore
where
    P: AsRef<Path> + ?Sized,
{
    type Error = io::Error;
    type Reader = File;

    const KIND: &'static str = "Filesystem";

    fn fetch(&mut self, id: &P) -> Result<File, io::Error> {
        self.find(id)
    }
}
