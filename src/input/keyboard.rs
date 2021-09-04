use bitflags::bitflags;

use crate::Context;

/// A key code.
pub use miniquad::KeyCode;
use std::collections::HashSet;

/// Tracks held down keyboard keys, active keyboard modifiers,
/// and figures out if the system is sending repeat keystrokes.
#[derive(Clone, Debug)]
pub struct KeyboardContext {
    active_modifiers: KeyMods,
    /// A simple mapping of which key code has been pressed.
    /// We COULD use a `Vec<bool>` but turning Rust enums to and from
    /// integers is unsafe and a set really is what we want anyway.
    pressed_keys_set: HashSet<KeyCode>,

    // These two are necessary for tracking key-repeat.
    last_pressed: Option<KeyCode>,
    current_pressed: Option<KeyCode>,
}

impl KeyboardContext {
    pub(crate) fn new() -> Self {
        Self {
            active_modifiers: KeyMods::empty(),
            // We just use 256 as a number Big Enough For Keyboard Keys to try to avoid resizing.
            pressed_keys_set: HashSet::with_capacity(256),
            last_pressed: None,
            current_pressed: None,
        }
    }

    pub(crate) fn set_key(&mut self, key: KeyCode, pressed: bool) {
        if pressed {
            let _ = self.pressed_keys_set.insert(key);
            self.last_pressed = self.current_pressed;
            self.current_pressed = Some(key);
        } else {
            let _ = self.pressed_keys_set.remove(&key);
            self.current_pressed = None;
        }

        self.set_key_modifier(key, pressed);
    }

    /// Take a modifier key code and alter our state.
    ///
    /// Double check that this edge handling is necessary;
    /// winit sounds like it should do this for us,
    /// see https://docs.rs/winit/0.18.0/winit/struct.KeyboardInput.html#structfield.modifiers
    ///
    /// ...more specifically, we should refactor all this to consistant-ify events a bit and
    /// make winit do more of the work.
    /// But to quote Scott Pilgrim, "This is... this is... Booooooring."
    fn set_key_modifier(&mut self, key: KeyCode, pressed: bool) {
        if pressed {
            match key {
                KeyCode::LeftShift | KeyCode::RightShift => self.active_modifiers |= KeyMods::SHIFT,
                KeyCode::LeftControl | KeyCode::RightControl => {
                    self.active_modifiers |= KeyMods::CTRL
                }
                KeyCode::LeftAlt | KeyCode::RightAlt => self.active_modifiers |= KeyMods::ALT,
                KeyCode::LeftSuper | KeyCode::RightSuper => self.active_modifiers |= KeyMods::LOGO,
                _ => (),
            }
        } else {
            match key {
                KeyCode::LeftShift | KeyCode::RightShift => self.active_modifiers -= KeyMods::SHIFT,
                KeyCode::LeftControl | KeyCode::RightControl => {
                    self.active_modifiers -= KeyMods::CTRL
                }
                KeyCode::LeftAlt | KeyCode::RightAlt => self.active_modifiers -= KeyMods::ALT,
                KeyCode::LeftSuper | KeyCode::RightSuper => self.active_modifiers -= KeyMods::LOGO,
                _ => (),
            }
        }
    }

    //pub(crate) fn set_modifiers(&mut self, keymods: KeyMods) {
    //    self.active_modifiers = keymods;
    //}

    pub(crate) fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys_set.contains(&key)
    }

    pub(crate) fn is_key_repeated(&self) -> bool {
        if self.last_pressed.is_some() {
            self.last_pressed == self.current_pressed
        } else {
            false
        }
    }

    pub(crate) fn pressed_keys(&self) -> &HashSet<KeyCode> {
        &self.pressed_keys_set
    }

    pub(crate) fn active_mods(&self) -> KeyMods {
        self.active_modifiers
    }
}

impl Default for KeyboardContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Checks if a key is currently pressed down.
pub fn is_key_pressed(ctx: &Context, key: KeyCode) -> bool {
    ctx.keyboard_context.is_key_pressed(key)
}

/// Checks if the last keystroke sent by the system is repeated, like when a key is held down for a period of time.
pub fn is_key_repeated(ctx: &Context) -> bool {
    ctx.keyboard_context.is_key_repeated()
}

/// Returns a reference to the set of currently pressed keys.
pub fn pressed_keys(ctx: &Context) -> &HashSet<KeyCode> {
    ctx.keyboard_context.pressed_keys()
}

/// Returns currently active keyboard modifiers.
pub fn active_mods(ctx: &Context) -> KeyMods {
    ctx.keyboard_context.active_mods()
}

/// Checks if keyboard modifier (or several) is active.
pub fn is_mod_active(ctx: &Context, keymods: KeyMods) -> bool {
    ctx.keyboard_context.active_mods().contains(keymods)
}

bitflags! {
    /// Bitflags describing the state of keyboard modifiers, such as `Control` or `Shift`.
    #[derive(Default)]
    pub struct KeyMods: u8 {
        /// No modifiers; equivalent to `KeyMods::default()` and
        /// [`KeyMods::empty()`](struct.KeyMods.html#method.empty).
        const NONE  = 0b0000_0000;
        /// Left or right Shift key.
        const SHIFT = 0b0000_0001;
        /// Left or right Control key.
        const CTRL  = 0b0000_0010;
        /// Left or right Alt key.
        const ALT   = 0b0000_0100;
        /// Left or right Win/Cmd/equivalent key.
        const LOGO  = 0b0000_1000;
    }
}

impl From<miniquad::KeyMods> for KeyMods {
    fn from(quad_mods: miniquad::KeyMods) -> Self {
        let mut keymods = KeyMods::NONE;

        if quad_mods.shift {
            keymods |= KeyMods::SHIFT;
        }
        if quad_mods.ctrl {
            keymods |= KeyMods::CTRL;
        }
        if quad_mods.alt {
            keymods |= KeyMods::ALT;
        }
        if quad_mods.logo {
            keymods |= KeyMods::LOGO;
        }

        keymods
    }
}
