use super::input_handler::InputHandler;
use crate::Context;

use crate::graphics::Point2;
pub use crate::input::input_handler::MouseButton;

pub struct MouseContext {
    pub(crate) input_handler: InputHandler,
    last_position: Point2,
    delta: Point2,
    cursor_grabbed: bool,
    cursor_hidden: bool,
    cursor_type: miniquad::CursorIcon,
}

impl MouseContext {
    pub fn new(input_handler: InputHandler) -> Self {
        MouseContext {
            input_handler,
            last_position: Point2::new(0., 0.),
            delta: Point2::new(0., 0.),
            cursor_grabbed: false,
            cursor_hidden: false,
            cursor_type: miniquad::CursorIcon::Default,
        }
    }

    pub(crate) fn set_last_position(&mut self, p: Point2) {
        self.last_position = p;
    }

    /// Resets the value returned by [`mouse::delta`](fn.delta.html) to zero.
    /// You shouldn't need to call this, except when you're running your own event loop.
    /// In this case call it right at the end, after `draw` and `update` have finished.
    pub fn reset_delta(&mut self) {
        self.delta = Point2::new(0., 0.);
    }

    pub(crate) fn set_delta(&mut self, p: Point2) {
        self.delta = p;
    }

    pub fn mouse_position(&self) -> cgmath::Point2<f32> {
        self.input_handler.mouse_position
    }

    pub fn button_pressed(&self, button: MouseButton) -> bool {
        self.input_handler.is_mouse_key_down(&button)
    }

    pub fn wheel(&self) -> f32 {
        self.input_handler.wheel
    }
}

/// The current mouse position in pixels.
pub fn position(ctx: &Context) -> mint::Point2<f32> {
    ctx.mouse_context.mouse_position().into()
}

/// Whether a certain mouse button is currently pressed.
pub fn button_pressed(ctx: &Context, button: MouseButton) -> bool {
    ctx.mouse_context.button_pressed(button)
}

pub fn wheel(ctx: &Context) -> f32 {
    ctx.mouse_context.wheel()
}

/// Get the distance the cursor was moved during the current frame, in pixels.
pub fn delta(ctx: &Context) -> mint::Point2<f32> {
    ctx.mouse_context.delta.into()
}

/// The position the mouse had during the latest `mouse_motion_event`
pub fn last_position(ctx: &Context) -> mint::Point2<f32> {
    ctx.mouse_context.last_position.into()
}

/// Get whether or not the mouse is grabbed (confined to the window)
pub fn cursor_grabbed(ctx: &Context) -> bool {
    ctx.mouse_context.cursor_grabbed
}

/// Set whether or not the mouse is grabbed (confined to the window)
pub fn set_cursor_grabbed(
    ctx: &mut Context,
    quad_ctx: &mut miniquad::GraphicsContext,
    grabbed: bool,
) {
    ctx.mouse_context.cursor_grabbed = grabbed;
    quad_ctx.set_cursor_grab(grabbed);
}

/// Returns the current mouse cursor type of the window.
pub fn cursor_type(ctx: &Context) -> miniquad::CursorIcon {
    ctx.mouse_context.cursor_type
}

/// Modifies the mouse cursor type of the window.
pub fn set_cursor_type(
    ctx: &mut Context,
    quad_ctx: &mut miniquad::graphics::GraphicsContext,
    cursor_type: miniquad::CursorIcon,
) {
    ctx.mouse_context.cursor_type = cursor_type;
    quad_ctx.set_mouse_cursor(cursor_type);
}

/// Set whether or not the mouse is hidden (invisible)
pub fn cursor_hidden(ctx: &Context) -> bool {
    ctx.mouse_context.cursor_hidden
}

/// Set whether or not the mouse is hidden (invisible).
pub fn set_cursor_hidden(
    ctx: &mut Context,
    quad_ctx: &mut miniquad::graphics::GraphicsContext,
    hidden: bool,
) {
    ctx.mouse_context.cursor_hidden = hidden;
    quad_ctx.show_mouse(!hidden);
}
