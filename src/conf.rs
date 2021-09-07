use std::path::PathBuf;

#[derive(Debug)]
pub enum Cache {
    /// No preloading at all, filesystem::open will always panic.
    No,
    /// Load /index.txt first, and cache all the files specified.
    /// Game will not start until all the files will be cached
    Index,
    /// Same as Index, but with the files list instead of index.txt
    List(Vec<&'static str>),
    /// All the files in one tar archive
    Tar(Vec<u8>),
}

#[derive(Debug)]
pub enum Loading {
    /// No progressbar at all, no html special requirements
    No,
    /// Will look for some specific html elements and show default progress bar
    Embedded,
}

#[derive(Debug)]
pub struct Conf {
    pub cache: Cache,
    pub loading: Loading,
    /// `Filesystem::open` will try to read from this dir if there's no such file in the cache.
    ///
    /// Note that this won't work on platforms where `std::fs` is unavailable, like WASM.
    pub physical_root_dir: Option<PathBuf>,
}

impl Default for Conf {
    fn default() -> Conf {
        Conf {
            cache: Cache::No,
            loading: Loading::No,
            physical_root_dir: None,
        }
    }
}

/// The possible number of samples for multisample anti-aliasing.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NumSamples {
    /// Multisampling disabled.
    Zero = 0,
    /// One sample
    One = 1,
    /// Two samples
    Two = 2,
    /// Four samples
    Four = 4,
    /// Eight samples
    Eight = 8,
    /// Sixteen samples
    Sixteen = 16,
}
