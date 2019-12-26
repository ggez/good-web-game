use std::rc::Rc;

use crate::conf::Loading;

pub trait LoadingPage: std::fmt::Debug {
    /// Will be called when all the html/wasm will be loaded and the filesystem is going to start download files
    fn show(&self);
    /// May be called without show - to hide embedded in html default progress bar.
    fn hide(&self);
    fn update_progress(&self, progress: f32);
}

#[derive(Debug)]
struct EmptyLoadingPage;
impl LoadingPage for EmptyLoadingPage {
    fn show(&self) {}
    fn hide(&self) {}
    fn update_progress(&self, _progress: f32) {}
}

#[derive(Debug)]
struct EmbeddedProgressBar;
impl EmbeddedProgressBar {
    fn progress_string(progress: f32) -> String {
        let a = progress * 10.;
        let b = 10. - progress * 10. + 1.;
        format!(
            "[ {} ]",
            std::iter::repeat('|')
                .take(a as usize)
                .chain(std::iter::repeat('-').take(b as usize))
                .collect::<String>()
        )
    }
}
impl LoadingPage for EmbeddedProgressBar {
    fn show(&self) {
        unimplemented!()
    }

    fn hide(&self) {
        unimplemented!()
    }

    fn update_progress(&self, progress: f32) {
        unimplemented!()
    }
}

pub fn from_conf(loading: Loading) -> Rc<dyn LoadingPage> {
    match loading {
        Loading::No => Rc::new(EmptyLoadingPage),
        Loading::Embedded => Rc::new(EmbeddedProgressBar),
        Loading::Custom(loading) => Rc::from(loading),
    }
}
