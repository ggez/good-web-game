mod canvas;
mod context;
mod drawparam;
mod image;
mod shader;
mod text;
mod types;

pub mod spritebatch;

use crate::error::GameResult;
use crate::Context;
pub(crate) use context::GpuText;

pub use self::{
    canvas::{set_canvas, Canvas},
    context::GraphicsContext,
    drawparam::DrawParam,
    image::*,
    shader::*,
    text::*,
    types::*,
};
use miniquad::PassAction;

/// Clear the screen to the background color.
pub fn clear(ctx: &mut Context, color: Color) {
    let action = PassAction::Clear {
        color: Some((color.r, color.g, color.b, color.a)),
        depth: None,
        stencil: None,
    };

    let pass = ctx.framebuffer();
    ctx.quad_ctx.begin_pass(pass, action);
    ctx.quad_ctx
        .clear(Some((color.r, color.g, color.b, color.a)), None, None);
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
    let gfx = &mut context.internal.gfx_context;
    gfx.set_transform(transform);
}

pub fn push_transform(context: &mut Context, transform: &cgmath::Matrix3<f32>) {
    let gfx = &mut context.internal.gfx_context;
    gfx.push_transform(transform);
}

/// Returns the size of the window in pixels as (width, height),
/// including borders, titlebar, etc.
/// Returns zeros if the window doesn't exist.
/// TODO: Rename, since get_drawable_size is usually what we
/// actually want. Maybe get_entire_size or get_window_border_size?
pub fn size(ctx: &Context) -> (f32, f32) {
    let size = ctx.quad_ctx.screen_size();
    (size.0, size.1)
}

/// Returns the size of the window's underlying drawable in pixels as (width, height).
/// This may return a different value than `get_size()` when run on a platform with high-DPI support
pub fn drawable_size(ctx: &Context) -> (u32, u32) {
    let size = ctx.quad_ctx.screen_size();
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
    context.internal.gfx_context.set_screen_coordinates(rect);
    Ok(())
}

/// Returns a rectangle defining the coordinate system of the screen.
/// It will be `Rect { x: left, y: top, w: width, h: height }`
///
/// If the Y axis increases downwards, the `height` of the `Rect`
/// will be negative.
pub fn screen_coordinates(ctx: &Context) -> Rect {
    ctx.internal.gfx_context.screen_rect
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

/// Applies `DrawParam` to `Rect`.
pub fn transform_rect(rect: Rect, param: DrawParam) -> Rect {
    let w = param.src.w * param.scale.x * rect.w;
    let h = param.src.h * param.scale.y * rect.h;
    let offset_x = w * param.offset.x;
    let offset_y = h * param.offset.y;
    let dest_x = param.dest.x - offset_x;
    let dest_y = param.dest.y - offset_y;
    let mut r = Rect {
        w,
        h,
        x: dest_x + rect.x * param.scale.x,
        y: dest_y + rect.y * param.scale.y,
    };
    r.rotate(param.rotation);
    r
}
