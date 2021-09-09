mod canvas;
mod context;
mod drawparam;
mod image;
#[cfg(feature = "mesh")]
mod mesh;
mod shader;
mod text;
mod types;

pub mod spritebatch;

use crate::error::GameResult;
use crate::Context;

pub use self::{
    canvas::{set_canvas, Canvas},
    context::GraphicsContext,
    drawparam::DrawParam,
    image::*,
    shader::*,
    text::*,
    types::*,
};

#[cfg(feature = "mesh")]
pub use self::mesh::*;

use crate::graphics::drawparam::Transform;
use miniquad::PassAction;
//use miniquad_text_rusttype::{FontTexture, TextDisplay};

/// Holds the bindings of objects that were dropped this frame.
/// They (and the buffers inside of them) are kept alive until the beginning of the next frame
/// to ensure that they're not deleted before being used in the frame in which they were dropped.
static mut DROPPED_BINDINGS: Vec<miniquad::Bindings> = Vec::new();
//type TextDisp = TextDisplay<std::rc::Rc<FontTexture>>;
/// The same as `DROPPED_BINDINGS`, but for text, as we can't access their internal bindings directly.
//static mut DROPPED_TEXT: Vec<TextDisp> = Vec::new();

/// Adds some bindings to a vec where they'll be kept alive until the beginning of the next frame.
pub(crate) fn add_dropped_bindings(bindings: miniquad::Bindings) {
    unsafe { DROPPED_BINDINGS.push(bindings) };
}
/// Adds some bindings to a vec where they'll be kept alive until the beginning of the next frame.
//pub(crate) fn add_dropped_text(text_disp: TextDisp) {
//    unsafe { DROPPED_TEXT.push(text_disp) };
//}

/// Deletes all buffers that were dropped in the previous frame and kept alive for its duration.
pub(crate) fn release_dropped_bindings() {
    unsafe {
        for bindings in DROPPED_BINDINGS.iter_mut() {
            for v_buffer in bindings.vertex_buffers.iter_mut() {
                v_buffer.delete();
            }
            bindings.index_buffer.delete();
        }
        DROPPED_BINDINGS.clear();
        // dropping the text is enough to trigger `TextDisplay::drop`, which deletes the buffers
        //DROPPED_TEXT.clear();
    }
}

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

pub fn set_projection<M>(context: &mut Context, proj: M)
where
    M: Into<mint::ColumnMatrix4<f32>>,
{
    let proj = cgmath::Matrix4::from(proj.into());
    let gfx = &mut context.gfx_context;
    gfx.set_projection(proj);
}

pub fn mul_projection<M>(context: &mut Context, proj: M)
where
    M: Into<mint::ColumnMatrix4<f32>>,
{
    let proj = cgmath::Matrix4::from(proj.into());
    let gfx = &mut context.gfx_context;
    let curr = gfx.projection();
    gfx.set_projection(proj * curr);
}

/// Returns the size of the window in pixels as (width, height),
/// including borders, titlebar, etc.
/// Returns zeros if the window doesn't exist.
pub fn size(_ctx: &Context) -> (f32, f32) {
    unimplemented!("use `drawable_size()` for getting the size of the underlying window's drawable")
}

/// Returns the size of the window's underlying drawable in pixels as (width, height).
/// This may return a different value than `get_size()` when run on a platform with high-DPI support
pub fn drawable_size(ctx: &Context) -> (f32, f32) {
    ctx.quad_ctx.screen_size()
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
    ctx.gfx_context.screen_rect
}

/// Tells the graphics system to actually put everything on the screen.
/// Call this at the end of your [`EventHandler`](../event/trait.EventHandler.html)'s
/// [`draw()`](../event/trait.EventHandler.html#tymethod.draw) method.
///
/// Unsets any active canvas.
pub fn present(ctx: &mut Context) -> GameResult<()> {
    crate::graphics::set_canvas(ctx, None);
    ctx.quad_ctx.commit_frame(); // TODO: replace this with an actual flush
    Ok(())
}
/*
pub fn set_font_size(ctx: &mut Context, font_size: u32) {
    ctx.gfx_context.font_size = font_size;
}
*/
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
    match param.trans {
        Transform::Values {
            scale,
            offset,
            dest,
            rotation,
        } => {
            // first apply the offset
            let mut r = Rect {
                w: rect.w,
                h: rect.h,
                x: rect.x - offset.x * rect.w,
                y: rect.y - offset.y * rect.h,
            };
            // apply the scale
            let real_scale = (param.src.w * scale.x, param.src.h * scale.y);
            r.w = real_scale.0 * rect.w;
            r.h = real_scale.1 * rect.h;
            r.x *= real_scale.0;
            r.y *= real_scale.1;
            // apply the rotation
            r.rotate(rotation);
            // apply the destination translation
            r.x += dest.x;
            r.y += dest.y;

            r
        }
        Transform::Matrix(_m) => todo!("Fix me"),
    }
}

#[cfg(test)]
mod tests {
    use crate::graphics::{transform_rect, DrawParam, Rect};
    use approx::assert_relative_eq;
    use std::f32::consts::PI;

    #[test]
    fn headless_test_transform_rect() {
        {
            let r = Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            };
            let param = DrawParam::default();
            let real = transform_rect(r, param);
            let expected = r;
            assert_relative_eq!(real, expected);
        }
        {
            let r = Rect {
                x: -1.0,
                y: -1.0,
                w: 2.0,
                h: 1.0,
            };
            let param = DrawParam::new().scale([0.5, 0.5]);
            let real = transform_rect(r, param);
            let expected = Rect {
                x: -0.5,
                y: -0.5,
                w: 1.0,
                h: 0.5,
            };
            assert_relative_eq!(real, expected);
        }
        {
            let r = Rect {
                x: -1.0,
                y: -1.0,
                w: 1.0,
                h: 1.0,
            };
            let param = DrawParam::new().offset([0.5, 0.5]);
            let real = transform_rect(r, param);
            let expected = Rect {
                x: -1.5,
                y: -1.5,
                w: 1.0,
                h: 1.0,
            };
            assert_relative_eq!(real, expected);
        }
        {
            let r = Rect {
                x: 1.0,
                y: 0.0,
                w: 2.0,
                h: 1.0,
            };
            let param = DrawParam::new().rotation(PI * 0.5);
            let real = transform_rect(r, param);
            let expected = Rect {
                x: -1.0,
                y: 1.0,
                w: 1.0,
                h: 2.0,
            };
            assert_relative_eq!(real, expected);
        }
        {
            let r = Rect {
                x: -1.0,
                y: -1.0,
                w: 2.0,
                h: 1.0,
            };
            let param = DrawParam::new()
                .scale([0.5, 0.5])
                .offset([0.0, 1.0])
                .rotation(PI * 0.5);
            let real = transform_rect(r, param);
            let expected = Rect {
                x: 0.5,
                y: -0.5,
                w: 0.5,
                h: 1.0,
            };
            assert_relative_eq!(real, expected);
        }
        {
            let r = Rect {
                x: -1.0,
                y: -1.0,
                w: 2.0,
                h: 1.0,
            };
            let param = DrawParam::new()
                .scale([0.5, 0.5])
                .offset([0.0, 1.0])
                .rotation(PI * 0.5)
                .dest([1.0, 0.0]);
            let real = transform_rect(r, param);
            let expected = Rect {
                x: 1.5,
                y: -0.5,
                w: 0.5,
                h: 1.0,
            };
            assert_relative_eq!(real, expected);
        }
        {
            let r = Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            };
            let param = DrawParam::new()
                .offset([0.5, 0.5])
                .rotation(PI * 1.5)
                .dest([1.0, 0.5]);
            let real = transform_rect(r, param);
            let expected = Rect {
                x: 0.5,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            };
            assert_relative_eq!(real, expected);
        }
        {
            let r = Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            };
            let param = DrawParam::new()
                .offset([0.5, 0.5])
                .rotation(PI * 0.25)
                .scale([2.0, 1.0])
                .dest([1.0, 2.0]);
            let real = transform_rect(r, param);
            let sqrt = (2f32).sqrt() / 2.;
            let unit = sqrt + sqrt / 2.;
            let expected = Rect {
                x: -unit + 1.,
                y: -unit + 2.,
                w: 2. * unit,
                h: 2. * unit,
            };
            assert_relative_eq!(real, expected);
        }
    }
}
