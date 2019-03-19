use std::cell::RefCell;
use std::rc::Rc;

use super::input_handler::InputHandler;
use crate::Context;

pub use crate::input::input_handler::MouseButton;

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

    pub fn button_pressed(&self, button: MouseButton) -> bool {
        self.input_handler.borrow().is_mouse_key_down(&button)
    }

    pub fn wheel(&self) -> f32 {
        self.input_handler.borrow().wheel
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
