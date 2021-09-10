use std::path::PathBuf;

#[derive(Debug)]
pub enum Loading {
    /// No progressbar at all, no html special requirements
    No,
    /// Will look for some specific html elements and show default progress bar
    Embedded,
}

#[derive(Debug)]
pub struct Conf {
    pub loading: Loading,
    /// `Filesystem::open` will try to read from this dir if there's no such file in the cache.
    ///
    /// Note that this won't work on platforms where `std::fs` is unavailable, like WASM.
    pub physical_root_dir: Option<PathBuf>,
}

impl Default for Conf {
    fn default() -> Conf {
        Conf {
            loading: Loading::No,
            physical_root_dir: None,
        }
    }
}

pub fn default_quad_conf() -> miniquad::conf::Conf {
    miniquad::conf::Conf {
        cache: miniquad::conf::Cache::No,
        window_title: "An easy, good game".to_string(),
        window_width: 800,
        window_height: 600,
        high_dpi: true,
        fullscreen: false,
        sample_count: 1,
        window_resizable: false
    }
}