use std::path::PathBuf;

/*
#[derive(Debug)]
pub enum Loading {
    /// No progressbar at all, no html special requirements
    No,
    /// Will look for some specific html elements and show default progress bar
    Embedded,
}
*/

/// Holds configuration values setting different options when starting the game, some of which
/// can't be changed later.
#[derive(Debug)]
pub struct Conf {
    //pub loading: Loading,
    /// `Filesystem::open` will try to read from this dir if there's no such file in the cache.
    ///
    /// Note that this won't work on platforms where `std::fs` is unavailable, like WASM.
    pub physical_root_dir: Option<PathBuf>,
    pub(crate) quad_conf: miniquad::conf::Conf,
}

impl Default for Conf {
    fn default() -> Conf {
        Conf {
            //loading: Loading::No,
            physical_root_dir: None,
            quad_conf: miniquad::conf::Conf {
                cache: miniquad::conf::Cache::No,
                window_title: "An easy, good game".to_string(),
                window_width: 800,
                window_height: 600,
                high_dpi: false,
                fullscreen: false,
                sample_count: 1,
                window_resizable: false,
            },
        }
    }
}

impl Conf {
    /*
    /// Whether to show a progress bar (doesn't do anything currently)
    pub fn loading(mut self, val: Loading) -> Self {
        self.loading = val;
        self
    }
    */
    /// Set the root of your filesystem.
    ///
    /// Default: `None`
    pub fn physical_root_dir(mut self, val: Option<PathBuf>) -> Self {
        self.physical_root_dir = val;
        self
    }
    /// Set the cache, holding embedded files for later use.
    ///
    /// Default: `miniquad::conf::Cache::No`
    pub fn cache(mut self, val: miniquad::conf::Cache) -> Self {
        self.quad_conf.cache = val;
        self
    }
    /// Set the window title
    ///
    /// Default: "An easy, good game"
    pub fn window_title(mut self, val: String) -> Self {
        self.quad_conf.window_title = val;
        self
    }
    /// Set the window width in logical pixels.
    ///
    /// Note: See [`high_dpi`](#method.high_dpi) for physical width.
    ///
    /// Default: `800`
    pub fn window_width(mut self, val: i32) -> Self {
        self.quad_conf.window_width = val;
        self
    }
    /// Set the window height in logical pixels.
    ///
    /// Note: See [`high_dpi`](#method.high_dpi) for physical height.
    ///
    /// Default: `600`
    pub fn window_height(mut self, val: i32) -> Self {
        self.quad_conf.window_height = val;
        self
    }
    /// Sets whether the rendering canvas is full-resolution on HighDPI displays.
    /// * If set to `false` the rendering canvas will be created with the logical window size and
    ///   then scaled when rendering, to account for the difference between logical and physical size.
    /// * If set to `true` the rendering canvas will be created with the physical window size, which
    ///   can differ from the logical window size due to high-dpi scaling, leading to your drawable space
    ///   possibly having a different size than specified.
    ///
    /// Default: `false`
    pub fn high_dpi(mut self, val: bool) -> Self {
        self.quad_conf.high_dpi = val;
        self
    }
    /// Set whether to run in fullscreen mode.
    ///
    /// Default: `false`
    pub fn fullscreen(mut self, val: bool) -> Self {
        self.quad_conf.fullscreen = val;
        self
    }
    /// Set how many samples should be used in MSAA.
    ///
    /// Default: `1`
    pub fn sample_count(mut self, val: i32) -> Self {
        self.quad_conf.sample_count = val;
        self
    }
    /// Set whether the window should be resizable by the user.
    ///
    /// Default: `false`
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
