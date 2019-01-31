use smart_default::SmartDefault;

use crate::goodies::loading_page::LoadingPage;

#[derive(Debug)]
pub enum Cache {
    /// No preloading at all, filesystem::open will always panic.
    No,
    /// Load /index.txt first, and cache all the files specified.
    /// Game will not start until all the files will be cached
    Index,
    /// Same as Index, but with the files list instead of index.txt
    List(Vec<&'static str>),
}

#[derive(Debug)]
pub enum Loading {
    /// No progressbar at all, no html special requirements
    No,
    /// Will look for some specific html elements and show default progress bar
    Embedded,
    /// All the html work deligated to custom LoadingPage
    Custom(Box<dyn LoadingPage>),
}

#[derive(SmartDefault, Debug)]
pub struct Conf {
    #[default(Cache::No)]
    pub cache: Cache,

    #[default(Loading::No)]
    pub loading: Loading,
}
