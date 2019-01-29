use smart_default::SmartDefault;

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

#[derive(SmartDefault, Debug)]
pub struct Conf {
    #[default(Cache::No)]
    pub cache: Cache,
}
