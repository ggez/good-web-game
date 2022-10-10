use miniquad::conf::{LinuxBackend, LinuxX11Gl, Platform};
use std::path::PathBuf;

/// Holds configuration values setting different options when starting the game, some of which
/// can't be changed later.
#[derive(Debug)]
pub struct Conf {
    /// `Filesystem::open` will try to read from this dir if there's no such file in the cache.
    ///
    /// Note that this won't work on platforms where `std::fs` is unavailable, like WASM.
    pub physical_root_dir: Option<PathBuf>,
    pub(crate) cache: Option<&'static [u8]>,
    pub(crate) quad_conf: miniquad::conf::Conf,
}

impl Default for Conf {
    fn default() -> Conf {
        Conf {
            physical_root_dir: None,
            cache: None,
            quad_conf: miniquad::conf::Conf {
                window_title: "An easy, good game".to_string(),
                window_width: 800,
                window_height: 600,
                high_dpi: false,
                fullscreen: false,
                sample_count: 1,
                window_resizable: false,
                icon: None,
                platform: Platform {
                    linux_x11_gl: LinuxX11Gl::GLXWithEGLFallback,
                    linux_backend: LinuxBackend::X11WithWaylandFallback,
                    swap_interval: None,
                    framebuffer_alpha: false,
                },
            },
        }
    }
}

impl Conf {
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
    pub fn cache(mut self, val: Option<&'static [u8]>) -> Self {
        self.cache = val;
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
