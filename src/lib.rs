pub mod conf;
pub mod error;
pub mod event;
pub mod filesystem;
pub mod goodies;
pub mod graphics;
pub mod input;
pub mod timer;

mod context;

pub use crate::{
    context::Context, error::GameError, error::GameResult, goodies::console,
    goodies::matrix_transform_2d,
};
pub use cgmath;

#[cfg(feature = "nalgebra")]
pub extern crate nalgebra;

pub fn start<F>(conf: conf::Conf, f: F) -> GameResult
where
    F: 'static + FnOnce(Context) -> GameResult,
{
    stdweb::initialize();

    filesystem::mount(conf.cache, conf.loading, |fs| {
        let context = Context::build(fs);
        f(context)
    });
    Ok(())
}
