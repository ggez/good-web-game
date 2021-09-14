use crate::context::Context;

pub use crate::input::keyboard::KeyMods;
pub use crate::input::MouseButton;
pub use miniquad::{KeyCode, TouchPhase};

/// Used in [`EventHandler`](trait.EventHandler.html)
/// to specify where an error originated
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ErrorOrigin {
    /// error originated in `update()`
    Update,
    /// error originated in `draw()`
    Draw,
}

/// A trait defining event callbacks.  This is your primary interface with
/// `ggez`'s event loop.  Implement this trait for a type and
/// override at least the [`update()`](#tymethod.update) and
/// [`draw()`](#tymethod.draw) methods, then pass it to
/// [`event::run()`](fn.run.html) to run the game's mainloop.
///
/// The default event handlers do nothing, apart from
/// [`key_down_event()`](#tymethod.key_down_event), which will by
/// default exit the game if the escape key is pressed.  Just
/// override the methods you want to use.
pub trait EventHandler<E>
where
    E: std::error::Error,
{
    /// Called upon each logic update to the game.
    /// This should be where the game's logic takes place.
    fn update(&mut self, _ctx: &mut Context) -> Result<(), E>;
    /// Called to do the drawing of your game.
    /// You probably want to start this with
    /// [`graphics::clear()`](../graphics/fn.clear.html) and end it
    /// with [`graphics::present()`](../graphics/fn.present.html).
    fn draw(&mut self, _ctx: &mut Context) -> Result<(), E>;
    /// Called when the user resizes the window, or when it is resized
    /// via [`graphics::set_drawable_size()`](../graphics/fn.set_drawable_size.html).
    fn resize_event(&mut self, _ctx: &mut Context, _width: f32, _height: f32) {}
    /// The mouse was moved; it provides both absolute x and y coordinates in the window,
    /// and relative x and y coordinates compared to its last position.
    fn mouse_motion_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32, _dx: f32, _dy: f32) {}
    fn touch_event(&mut self, ctx: &mut Context, phase: TouchPhase, _id: u64, x: f32, y: f32) {
        if phase == TouchPhase::Started {
            self.mouse_button_down_event(ctx, MouseButton::Left, x, y);
        }

        if phase == TouchPhase::Ended {
            self.mouse_button_up_event(ctx, MouseButton::Left, x, y);
        }
    }
    /// The mousewheel was scrolled, vertically (y, positive away from and negative toward the user)
    /// or horizontally (x, positive to the right and negative to the left).
    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32) {}
    /// A mouse button was pressed.
    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
    }
    /// A mouse button was released.
    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
    }

    /// A keyboard button was pressed.
    ///
    /// The default implementation of this will call `ggez::event::quit()`
    /// when the escape key is pressed.  If you override this with
    /// your own event handler you have to re-implment that
    /// functionality yourself.
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        if keycode == KeyCode::Escape {
            quit(ctx);
        }
    }

    /// A keyboard button was released.
    fn key_up_event(&mut self, _ctx: &mut Context, _keycode: KeyCode, _keymods: KeyMods) {}

    /// A unicode character was received, usually from keyboard input.
    /// This is the intended way of facilitating text input.
    fn text_input_event(&mut self, _ctx: &mut Context, _character: char) {}

    /// Something went wrong, causing a `GameError`.
    /// If this returns true, the error was fatal, so the event loop ends, aborting the game.
    fn on_error(&mut self, _ctx: &mut Context, _origin: ErrorOrigin, _e: E) -> bool {
        true
    }
}

/// Terminates the [`ggez::event::run()`](fn.run.html) loop by setting
/// [`Context.continuing`](struct.Context.html#structfield.continuing)
/// to `false`.
pub fn quit(ctx: &mut Context) {
    ctx.continuing = false;
}
