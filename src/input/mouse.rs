use std::cell::RefCell;
use std::rc::Rc;

use super::input_handler::InputHandler;
use crate::Context;

pub struct MouseContext {
    pub(crate) input_handler: Rc<RefCell<InputHandler>>,
}

impl MouseContext {
    pub fn new(input_handler: Rc<RefCell<InputHandler>>) -> Self {
        MouseContext { input_handler }
    }

    pub fn mouse_position(&self) -> cgmath::Point2<f32> {
        self.input_handler
            .borrow()
            .mouse_position
            .cast::<f32>()
            .unwrap()
    }
}

/// Get the current position of the mouse cursor, in pixels.
/// Complement to [`set_position()`](fn.set_position.html).
/// Uses strictly window-only coordinates.
pub fn position(ctx: &Context) -> cgmath::Point2<f32> {
    ctx.mouse_context.mouse_position()
}
