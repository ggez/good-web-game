// large parts directly stolen from macroquad: https://github.com/not-fl3/macroquad/blob/854aa50302a00ce590d505e28c9ecc42ae24be58/src/file.rs

use std::{collections::HashMap, io, path};
use std::sync::{Arc, Mutex};

use crate::{conf::Conf, Context, GameError, GameResult};
use std::panic::panic_any;
use crate::GameError::ResourceLoadError;

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
    root: Option<path::PathBuf>,
    files: HashMap<path::PathBuf, File>,
}

impl Filesystem {
    pub(crate) fn new(conf: &Conf) -> Filesystem {
        let mut files = HashMap::new();

        if let Some(tar_file) = conf.cache {
            let mut archive = tar::Archive::new(tar_file);

            for file in archive.entries().unwrap_or_else(|e| panic_any(e)) {
                use std::io::Read;

                let mut file = file.unwrap_or_else(|e| panic_any(e));
                let filename =
                    std::path::PathBuf::from(file.path().unwrap_or_else(|e| panic_any(e)));
                let mut buf = vec![];

                file.read_to_end(&mut buf).unwrap_or_else(|e| panic_any(e));
                if !buf.is_empty() {
                    files.insert(
                        filename,
                        File {
                            bytes: io::Cursor::new(buf),
                        },
                    );
                }
            }
        }

        let root = conf.physical_root_dir.clone();
        Filesystem { root, files }
    }

    /// Opens the given `path` and returns the resulting `File`
    /// in read-only mode.
    pub fn open<P: AsRef<path::Path>>(&mut self, path: P) -> GameResult<File> {
        let mut path = path::PathBuf::from(path.as_ref());

        // workaround for ggez-style pathes: in ggez paths starts with "/", while in the cache
        // dictionary they are presented without "/"
        if let Ok(stripped) = path.strip_prefix("/") {
            path = path::PathBuf::from(stripped);
        }

        // first check the cache
        if self.files.contains_key(&path) {
            Ok(self.files[&path].clone())
        } else {
            // the file is not inside the cache, so it has to be loaded (locally, or via http url)
            let file = self.load_file(&path)?;
            Ok(file)
        }

    }

    /// Load file from the path and block until its loaded
    /// Will use filesystem on PC and do http request on web
    fn load_file<P: AsRef<path::Path>>(&self, path: P) -> GameResult<File> {
        fn load_file_inner(path: &str) -> GameResult<Vec<u8>> {

            let contents = Arc::new(Mutex::new(None));

            {
                let contents = contents.clone();
                let err_path = path.to_string();

                miniquad::fs::load_file(path, move |bytes| {
                    *contents.lock().unwrap() =
                        Some(bytes.map_err(|kind| GameError::ResourceLoadError(format!("Couldn't load file {}: {}", err_path, kind))));
                });
            }

            // wait until the file has been loaded
            // as miniquad::fs::load_file internally uses non-asynchronous loading for everything
            // except wasm, waiting should only ever occur on wasm
            loop {
                let mut contents_guard = contents.lock().unwrap();
                if let Some(contents) = contents_guard.take() {
                    return contents;
                }
                drop(contents_guard);
                std::thread::yield_now();
            }
        }

        #[cfg(target_os = "ios")]
            let _ = std::env::set_current_dir(std::env::current_exe().unwrap().parent().unwrap());

        let path = path.as_ref().as_os_str().to_os_string().into_string().map_err(|os_string| ResourceLoadError(format!("utf-8-invalid path: {:?}", os_string)))?;

        #[cfg(not(target_os = "android"))]
            let path = if let Some(ref root) = self.root {
            format!("{}/{}",
                    root.as_os_str()
                        .to_os_string()
                        .into_string()
                        .map_err(|os_string| ResourceLoadError(format!("utf-8-invalid root: {:?}", os_string)))?,
                    path)
        } else {
            path
        };

        println!("loading with path {}", &path);

        let buf = load_file_inner(&path)?;
        let bytes = io::Cursor::new(buf);
        Ok(File { bytes })
    }
}

/// Opens the given path and returns the resulting `File`
/// in read-only mode.
pub fn open<P: AsRef<path::Path>>(ctx: &mut Context, path: P) -> GameResult<File> {
    ctx.filesystem.open(path)
}
