pub(crate) mod input_handler;

#[cfg(not(any(target_arch = "wasm32", target_os = "ios", target_os = "android",)))]
pub mod gamepad;

pub mod keyboard;
pub mod mouse;

pub use self::{input_handler::MouseButton, keyboard::KeyboardContext, mouse::MouseContext};
