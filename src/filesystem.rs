use std::{collections::HashMap, io, path};

use crate::{Context, GameResult};

mod preload;

pub(crate) use self::preload::mount;

#[derive(Debug, Clone)]
pub enum File {
    Image(stdweb::web::html_element::ImageElement),
    Bytes(io::Cursor<Vec<u8>>),
}

impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            File::Image(_) => unimplemented!("read for images is not implemented yet!"),
            File::Bytes(b) => b.read(buf),
        }
    }
}

/// A structure that contains the filesystem state and cache.
#[derive(Debug)]
pub struct Filesystem {
    files: HashMap<path::PathBuf, File>,
}

impl Filesystem {
    pub(crate) fn new() -> Self {
        Filesystem {
            files: HashMap::new(),
        }
    }

    /// Opens the given `path` and returns the resulting `File`
    /// in read-only mode.
    pub fn open<P: AsRef<path::Path>>(&mut self, path: P) -> GameResult<File> {
        Ok(self.files[path.as_ref()].clone())
    }
}

pub fn open<P: AsRef<path::Path>>(ctx: &mut Context, path: P) -> GameResult<File> {
    ctx.filesystem.open(path)
}
