mod context;
mod drawparam;
mod image;
mod shader;
mod text;
mod types;

use crate::error::GameResult;
use crate::Context;

pub use self::{
    context::{canvas::*, CanvasContext, GraphicsContext, WebGlContext},
    drawparam::DrawParam,
    image::*,
    shader::*,
    text::*,
    types::*,
};

/// Clear the screen to the background color.
pub fn clear(ctx: &mut Context, color: Color) {
    ctx.gfx_context.clear(color);
}

/// Draws the given `Drawable` object to the screen by calling its
/// [`draw()`](trait.Drawable.html#tymethod.draw) method.
pub fn draw<D, T>(ctx: &mut Context, drawable: &D, params: T) -> GameResult
where
    D: Drawable,
    T: Into<DrawParam>,
{
    let params = params.into();
    drawable.draw(ctx, params)
}

pub fn set_transform(context: &mut Context, transform: &cgmath::Matrix3<f32>) {
    let gfx = &mut context.gfx_context;
    gfx.set_transform(transform);
}

pub fn push_transform(context: &mut Context, transform: &cgmath::Matrix3<f32>) {
    let gfx = &mut context.gfx_context;
    gfx.push_transform(transform);
}

/// Returns the size of the window in pixels as (width, height),
/// including borders, titlebar, etc.
/// Returns zeros if the window doesn't exist.
/// TODO: Rename, since get_drawable_size is usually what we
/// actually want. Maybe get_entire_size or get_window_border_size?
pub fn size(ctx: &Context) -> (f64, f64) {
    ctx.gfx_context.size()
}

/// Returns the size of the window's underlying drawable in pixels as (width, height).
/// This may return a different value than `get_size()` when run on a platform with high-DPI support
pub fn drawable_size(ctx: &Context) -> (u32, u32) {
    let size = ctx.gfx_context.size();
    (size.0 as u32, size.1 as u32)
}

/// Sets the bounds of the screen viewport.
///
/// The default coordinate system has (0,0) at the top-left corner
/// with X increasing to the right and Y increasing down, with the
/// viewport scaled such that one coordinate unit is one pixel on the
/// screen.  This function lets you change this coordinate system to
/// be whatever you prefer.
///
/// The `Rect`'s x and y will define the top-left corner of the screen,
/// and that plus its w and h will define the bottom-right corner.
pub fn set_screen_coordinates(context: &mut Context, rect: Rect) -> GameResult {
    context.gfx_context.set_screen_coordinates(rect);
    Ok(())
}

/// Returns a rectangle defining the coordinate system of the screen.
/// It will be `Rect { x: left, y: top, w: width, h: height }`
///
/// If the Y axis increases downwards, the `height` of the `Rect`
/// will be negative.
pub fn screen_coordinates(ctx: &Context) -> Rect {
    ctx.gfx_context.webgl_context.screen_rect
}

/// Tells the graphics system to actually put everything on the screen.
/// Call this at the end of your [`EventHandler`](../event/trait.EventHandler.html)'s
/// [`draw()`](../event/trait.EventHandler.html#tymethod.draw) method.
///
/// Unsets any active canvas.
pub fn present(_: &mut Context) -> GameResult<()> {
    Ok(())
}

/// All types that can be drawn on the screen implement the `Drawable` trait.
pub trait Drawable {
    /// Draws the drawable onto the rendering target.
    ///
    /// ALSO TODO: Expand docs
    fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult;

    /// Sets the blend mode to be used when drawing this drawable.
    /// This overrides the general [`graphics::set_blend_mode()`](fn.set_blend_mode.html).
    /// If `None` is set, defers to the blend mode set by
    /// `graphics::set_blend_mode()`.
    fn set_blend_mode(&mut self, mode: Option<BlendMode>);

    /// Gets the blend mode to be used when drawing this drawable.
    fn blend_mode(&self) -> Option<BlendMode>;

    fn dimensions(&self, _: &mut Context) -> Option<Rect> {
        None
    }
}
