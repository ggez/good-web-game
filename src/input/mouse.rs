use super::input_handler::InputHandler;
use crate::Context;

use crate::graphics::Point2;
pub use crate::input::input_handler::MouseButton;

pub struct MouseContext {
    pub(crate) input_handler: InputHandler,
    last_position: Point2,
    delta: Point2,
}

impl MouseContext {
    pub fn new(input_handler: InputHandler) -> Self {
        MouseContext {
            input_handler,
            last_position: Point2::new(0., 0.),
            delta: Point2::new(0., 0.),
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
        self.input_handler.mouse_position.cast::<f32>().unwrap()
    }

    pub fn button_pressed(&self, button: MouseButton) -> bool {
        self.input_handler.is_mouse_key_down(&button)
    }

    pub fn wheel(&self) -> f32 {
        self.input_handler.wheel
    }
}

pub fn position(ctx: &Context) -> cgmath::Point2<f32> {
    ctx.mouse_context.mouse_position()
}

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

pub fn last_position(ctx: &Context) -> mint::Point2<f32> {
    ctx.mouse_context.last_position.into()
}
