use bitflags::bitflags;
use std::cell::RefCell;
use std::rc::Rc;

use super::input_handler::InputHandler;
use crate::Context;

pub struct KeyboardContext {
    pub input_handler: Rc<RefCell<InputHandler>>,
}

impl KeyboardContext {
    pub(crate) fn new(input_handler: Rc<RefCell<InputHandler>>) -> Self {
        KeyboardContext { input_handler }
    }

    pub(crate) fn is_key_pressed(&self, key: &str) -> bool {
        self.input_handler.borrow().is_key_pressed(key)
    }

    pub(crate) fn is_key_down(&self, key: &str) -> bool {
        self.input_handler.borrow().is_key_down(key)
    }
}

/// Checks if a key is currently pressed down.
pub fn is_key_pressed(ctx: &Context, key: &str) -> bool {
    ctx.internal.keyboard_context.is_key_pressed(key)
}

/// Checks if a key was pressed down on exectly this frame.
pub fn is_key_down(ctx: &Context, key: &str) -> bool {
    ctx.internal.keyboard_context.is_key_down(key)
}

bitflags! {
    pub struct KeyMods: u8 {
        const NONE = 0;
        const SHIFT = 1 << 0;
        const CTRL = 1 << 1;
        const ALT = 1 << 2;
        const LOGO = 1 << 3;
    }
}

impl KeyMods {
    // pub(crate) fn from_event<E: IKeyboardEvent>(event: &E) -> Self {
    //     let mut keymods = KeyMods::NONE;

    //     if event.shift_key() {
    //         keymods |= KeyMods::SHIFT;
    //     }
    //     if event.ctrl_key() {
    //         keymods |= KeyMods::CTRL;
    //     }
    //     if event.alt_key() {
    //         keymods |= KeyMods::ALT;
    //     }
    //     if event.meta_key() {
    //         keymods |= KeyMods::LOGO;
    //     }

    //     keymods
    // }
}
