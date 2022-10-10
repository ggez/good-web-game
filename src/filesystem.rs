// large parts directly stolen from macroquad: https://github.com/not-fl3/macroquad/blob/854aa50302a00ce590d505e28c9ecc42ae24be58/src/file.rs

use std::sync::{Arc, Mutex};
use std::{collections::HashMap, io, path};

use crate::GameError::ResourceLoadError;
use crate::{conf::Conf, Context, GameError, GameResult};
use std::panic::panic_any;

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
    #[allow(clippy::redundant_closure)]
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

    #[cfg(not(target_os = "wasm32"))]
    /// Load file from the path and block until its loaded
    /// Will use filesystem on PC and Android and fail on WASM
    fn load_file<P: AsRef<path::Path>>(&self, path: P) -> GameResult<File> {
        fn load_file_inner(path: &str) -> GameResult<Vec<u8>> {
            let contents = Arc::new(Mutex::new(None));

            {
                let contents = contents.clone();
                let err_path = path.to_string();

                miniquad::fs::load_file(path, move |bytes| {
                    *contents.lock().unwrap() = Some(bytes.map_err(|kind| {
                        GameError::ResourceLoadError(format!(
                            "Couldn't load file {}: {}",
                            err_path, kind
                        ))
                    }));
                });
            }

            // wait until the file has been loaded
            // as miniquad::fs::load_file internally uses non-asynchronous loading for everything
            // except wasm, waiting should only ever occur on wasm (TODO: since this holds the main
            // thread hostage no progress is ever made and this just blocks forever... perhaps this
            // could be worked around by using "asyncify", but that would be both hard and also
            // require an additional post processing step on the generated wasm file)
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

        let path = path
            .as_ref()
            .as_os_str()
            .to_os_string()
            .into_string()
            .map_err(|os_string| {
                ResourceLoadError(format!("utf-8-invalid path: {:?}", os_string))
            })?;

        #[cfg(not(target_os = "android"))]
        let path = if let Some(ref root) = self.root {
            format!(
                "{}/{}",
                root.as_os_str()
                    .to_os_string()
                    .into_string()
                    .map_err(|os_string| ResourceLoadError(format!(
                        "utf-8-invalid root: {:?}",
                        os_string
                    )))?,
                path
            )
        } else {
            path
        };

        let buf = load_file_inner(&path)?;
        let bytes = io::Cursor::new(buf);
        Ok(File { bytes })
    }

    #[cfg(target_os = "wasm32")]
    /// Load file from the path and block until its loaded
    /// Will use filesystem on PC and Android and fail on WASM
    fn load_file<P: AsRef<path::Path>>(&self, path: P) -> GameResult<File> {
        Err(GameError::ResourceLoadError(format!(
            "Couldn't load file {}",
            path.as_display()
        )))
    }
}

/// Opens the given path and returns the resulting `File`
/// in read-only mode.
///
/// Checks the [embedded tar file](../conf/struct.Conf.html#method.high_dpi), if there is one, first and if the file cannot be found there
/// continues to either load the file using the OS-filesystem, or just fail on WASM, as blocking loads
/// are impossible there.
pub fn open<P: AsRef<path::Path>>(ctx: &mut Context, path: P) -> GameResult<File> {
    ctx.filesystem.open(path)
}

/// Loads a file from the path returning an `Option` that will be `Some` once it has been loaded (or loading it failed).
/// Will use filesystem on PC and Android and a http request on WASM.
///
/// Note: Don't wait for the `Option` to become `Some` inside of a loop, as that would create an infinite loop
/// on WASM, where progress on the GET request can only be made _between_ frames of your application.
pub fn load_file_async<P: AsRef<path::Path>>(path: P) -> Arc<Mutex<Option<GameResult<File>>>> {
    // TODO: Create an example showcasing the use of this.
    let contents = Arc::new(Mutex::new(None));
    let path = path
        .as_ref()
        .as_os_str()
        .to_os_string()
        .into_string()
        .map_err(|os_string| ResourceLoadError(format!("utf-8-invalid path: {:?}", os_string)));

    if let Ok(path) = path {
        let contents = contents.clone();

        miniquad::fs::load_file(&*(path.clone()), move |response| {
            let result = match response {
                Ok(bytes) => Ok(File {
                    bytes: io::Cursor::new(bytes),
                }),
                Err(e) => Err(GameError::ResourceLoadError(format!(
                    "Couldn't load file {}: {}",
                    path, e
                ))),
            };
            *contents.lock().unwrap() = Some(result);
        });
    }

    contents
}
