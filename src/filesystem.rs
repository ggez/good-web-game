use std::{collections::HashMap, io, path};

use crate::{Context, GameResult};

#[derive(Debug, Clone)]
pub struct File {
    pub bytes: io::Cursor<Vec<u8>>,
}

impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.bytes.read(buf)
    }
}

/// A structure that contains the filesystem state and cache.
#[derive(Debug)]
pub struct Filesystem {
    files: HashMap<path::PathBuf, File>,
}

impl Filesystem {
    pub(crate) fn new(tar_file: &[u8]) -> Filesystem {
        let mut archive = tar::Archive::new(tar_file);

        let mut files = HashMap::new();
        for file in archive.entries().unwrap_or_else(|e| panic!(e)) {
            use std::io::Read;

            let mut file = file.unwrap_or_else(|e| panic!(e));
            let filename = std::path::PathBuf::from(file.path().unwrap_or_else(|e| panic!(e)));
            let mut buf = vec![];

            file.read_to_end(&mut buf).unwrap_or_else(|e| panic!(e));
            if buf.len() != 0 {
                files.insert(
                    filename,
                    File {
                        bytes: io::Cursor::new(buf),
                    },
                );
            }
        }

        Filesystem { files }
    }

    /// Opens the given `path` and returns the resulting `File`
    /// in read-only mode.
    pub fn open<P: AsRef<path::Path>>(&mut self, path: P) -> GameResult<File> {
        let mut path = path::PathBuf::from(path.as_ref());

        // workaround for ggez-style pathes: in ggez pathes starts with "/", while in the cache
        // dictionary they are presented without "/"
        if let Ok(stripped) = path.strip_prefix("/") {
            path = path::PathBuf::from(stripped);
        }

        if self.files.contains_key(&path) == false {
            panic!("No such file: {:?}", &path);
        }
        Ok(self.files[&path].clone())
    }
}

pub fn open<P: AsRef<path::Path>>(ctx: &mut Context, path: P) -> GameResult<File> {
    ctx.filesystem.open(path)
}
