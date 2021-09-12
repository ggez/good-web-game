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
    pub(crate) quad_conf: miniquad::conf::Conf,
}

impl Default for Conf {
    fn default() -> Conf {
        Conf {
            loading: Loading::No,
            physical_root_dir: None,
            quad_conf: miniquad::conf::Conf {
                cache: miniquad::conf::Cache::No,
                window_title: "An easy, good game".to_string(),
                window_width: 800,
                window_height: 600,
                high_dpi: true,
                fullscreen: false,
                sample_count: 1,
                window_resizable: false,
            },
        }
    }
}

impl Conf {
    pub fn loading(mut self, val: Loading) -> Self {
        self.loading = val;
        self
    }
    pub fn physical_root_dir(mut self, val: Option<PathBuf>) -> Self {
        self.physical_root_dir = val;
        self
    }
    pub fn cache(mut self, val: miniquad::conf::Cache) -> Self {
        self.quad_conf.cache = val;
        self
    }
    pub fn window_title(mut self, val: String) -> Self {
        self.quad_conf.window_title = val;
        self
    }
    pub fn window_width(mut self, val: i32) -> Self {
        self.quad_conf.window_width = val;
        self
    }
    pub fn window_height(mut self, val: i32) -> Self {
        self.quad_conf.window_height = val;
        self
    }
    pub fn high_dpi(mut self, val: bool) -> Self {
        self.quad_conf.high_dpi = val;
        self
    }
    pub fn fullscreen(mut self, val: bool) -> Self {
        self.quad_conf.fullscreen = val;
        self
    }
    pub fn sample_count(mut self, val: i32) -> Self {
        self.quad_conf.sample_count = val;
        self
    }
    pub fn window_resizable(mut self, val: bool) -> Self {
        self.quad_conf.window_resizable = val;
        self
    }
}

impl From<Conf> for miniquad::conf::Conf {
    fn from(conf: Conf) -> Self {
        conf.quad_conf
    }
}
