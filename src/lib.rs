//! # Good Web Game
//!
//! good-web-game is a wasm32-unknown-unknown implementation of a [ggez](https://github.com/ggez/ggez) subset
//! on top of [miniquad](https://github.com/not-fl3/miniquad/). Originally built to run [zemeroth](https://github.com/ozkriff/zemeroth) in the web.
//!
//! It has been recently updated to support much of the ggez 0.6.1 API. If you're already working with ggez
//! you might use this library to port your game to the web (or perhaps even mobile).
//! Since it also runs well on desktop it also offers an alternative implementation of ggez, which might
//! come in handy if you experience bugs in ggez, which you can't work around for some reason. Canvases
//! with multisampling are currently buggy in classic ggez while they work fine in good-web-game, for example.
//!
//! If you are looking for a properly maintained and supported minimal high-level engine on top of miniquad -
//! check out [macroquad](https://github.com/not-fl3/macroquad/) instead.
//!
//! ## Status
//!
//! "good-web-game" implements the most important parts of the ggez 0.6.1 API.
//!
//! ### Missing / Not available:
//!
//! * filesystem with writing access (if you need it take a look at [`quad-storage`](https://github.com/optozorax/quad-storage))
//! * game pad support
//! * writing your own event loop (doesn't make much sense on callback-only platforms like HTML5)
//! * spatial audio (overall audio support is still relatively limited, but could be improved)
//! * resolution control in fullscreen mode
//! * setting window position / size (the latter is available on Windows, but buggy)
//! * screenshot function
//! * window icon
//! * and custom shader support (yes, this is a big one, but if you need it and are familiar with `miniquad` please
//!   consider starting a PR; `miniquad` has all the tools you need)
//!
//!
//! ## Demo
//!
//! In action(0.1, pre-miniquad version): <https://ozkriff.itch.io/zemeroth>
//!
//! ![screen](https://i.imgur.com/TjvCNwa.jpg)
//!
//! ## Example
//!
//! To build and run an example as a native binary:
//!
//! ```rust
//! cargo run --example astroblasto
//! ```
//!
//! Building for web and mobile is currently a WIP (ironic, I know).
//! If you want to try your luck anyway the [miniquad instructions for WASM](https://github.com/not-fl3/miniquad/#wasm)
//! might be a good place to start.
//!
//! ## Architecture
//!
//! Here is how `good-web-game` fits into your rust-based game:
//!
//! ![software stack](about/gwg-stack.png?raw=true "good-web-game software stack")

pub mod audio;
pub mod conf;
pub mod error;
pub mod event;
pub mod filesystem;
pub mod goodies;
pub mod graphics;
pub mod input;
pub mod timer;

mod context;

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

pub use crate::context::Context;
pub use crate::error::*;

pub use cgmath;
pub extern crate mint;

use crate::event::ErrorOrigin;
use crate::filesystem::Filesystem;
#[cfg(not(any(target_arch = "wasm32", target_os = "ios", target_os = "android",)))]
use crate::input::gamepad::GamepadId;
use crate::input::mouse;
#[cfg(feature = "log-impl")]
pub use miniquad::{debug, info, log, warn};

struct EventHandlerWrapper<E: std::error::Error> {
    event_handler: Box<dyn event::EventHandler<E>>,
    context: Context,
}

impl<E: std::error::Error> miniquad::EventHandlerFree for EventHandlerWrapper<E> {
    fn update(&mut self) {
        // if the program is to quit, quit
        // (in ggez this is done before looking at any of the events of this frame, but this isn't
        //  possible here, so this is the closest it can get)
        if !self.context.continuing {
            self.context.quad_ctx.quit();
            // TODO: Even after this has been called the app might continue on executing into `EventHandler::update`
            //       starting another game logic cycle. This could be pretty bad...
            //       for now we somewhat fix this by yielding the time slice and simply returning
            std::thread::yield_now();
            return;
        }

        // in ggez tick is called before update, so I moved this to the front
        self.context.timer_context.tick();

        // release all buffers that were kept alive for the previous frame
        graphics::release_dropped_bindings();

        // before running the game logic update the gamepad state
        #[cfg(not(any(target_arch = "wasm32", target_os = "ios", target_os = "android",)))]
        {
            while let Some(gilrs::Event { id, event, .. }) =
                self.context.gamepad_context.next_event()
            {
                match event {
                    gilrs::EventType::ButtonPressed(button, _) => {
                        self.event_handler.gamepad_button_down_event(
                            &mut self.context,
                            button,
                            GamepadId(id),
                        );
                    }
                    gilrs::EventType::ButtonReleased(button, _) => {
                        self.event_handler.gamepad_button_up_event(
                            &mut self.context,
                            button,
                            GamepadId(id),
                        );
                    }
                    gilrs::EventType::AxisChanged(axis, value, _) => {
                        self.event_handler.gamepad_axis_event(
                            &mut self.context,
                            axis,
                            value,
                            GamepadId(id),
                        );
                    }
                    _ => {}
                }
            }
        }

        // do ggez 0.6 style error handling
        if let Err(e) = self.event_handler.update(&mut self.context) {
            error!("Error on EventHandler::update(): {:?}", e); // TODO: maybe use miniquad-logging here instead, but I haven't looked into it yet
            eprintln!("Error on EventHandler::update(): {:?}", e);
            if self
                .event_handler
                .on_error(&mut self.context, ErrorOrigin::Update, e)
            {
                event::quit(&mut self.context);
            }
        }
        if let Some(ref mut mixer) = &mut *self.context.audio_context.mixer.borrow_mut() {
            mixer.frame();
        }
    }

    fn draw(&mut self) {
        // do ggez 0.6 style error handling
        if let Err(e) = self.event_handler.draw(&mut self.context) {
            error!("Error on EventHandler::draw(): {:?}", e);
            eprintln!("Error on EventHandler::draw(): {:?}", e);
            if self
                .event_handler
                .on_error(&mut self.context, ErrorOrigin::Draw, e)
            {
                event::quit(&mut self.context);
            }
        }
        // reset the mouse frame delta value
        self.context.mouse_context.reset_delta();
    }

    fn resize_event(&mut self, width: f32, height: f32) {
        self.event_handler
            .resize_event(&mut self.context, width, height);
    }

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.context
            .mouse_context
            .input_handler
            .handle_mouse_move(x, y);
        let old_pos = mouse::last_position(&self.context);
        let dx = x - old_pos.x;
        let dy = y - old_pos.y;
        // update the frame delta value
        let old_delta = mouse::delta(&self.context);
        self.context
            .mouse_context
            .set_delta((old_delta.x + dx, old_delta.y + dy).into());
        self.context.mouse_context.set_last_position((x, y).into());
        self.event_handler
            .mouse_motion_event(&mut self.context, x, y, dx, dy);
    }

    fn mouse_button_down_event(&mut self, button: miniquad::MouseButton, x: f32, y: f32) {
        self.context
            .mouse_context
            .input_handler
            .handle_mouse_down(button);
        self.event_handler
            .mouse_button_down_event(&mut self.context, button.into(), x, y);
    }

    fn mouse_button_up_event(&mut self, button: miniquad::MouseButton, x: f32, y: f32) {
        self.context
            .mouse_context
            .input_handler
            .handle_mouse_up(button);
        self.event_handler
            .mouse_button_up_event(&mut self.context, button.into(), x, y);
    }

    fn char_event(&mut self, character: char, _keymods: miniquad::KeyMods, _repeat: bool) {
        self.event_handler
            .text_input_event(&mut self.context, character);
    }

    fn key_down_event(
        &mut self,
        keycode: miniquad::KeyCode,
        keymods: miniquad::KeyMods,
        repeat: bool,
    ) {
        // first update the keyboard context state
        self.context.keyboard_context.set_key(keycode, true);
        // then hand it to the user
        self.event_handler
            .key_down_event(&mut self.context, keycode, keymods.into(), repeat);
    }

    fn key_up_event(&mut self, keycode: miniquad::KeyCode, keymods: miniquad::KeyMods) {
        self.context.keyboard_context.set_key(keycode, false);
        self.event_handler
            .key_up_event(&mut self.context, keycode, keymods.into());
    }

    fn touch_event(&mut self, phase: miniquad::TouchPhase, id: u64, x: f32, y: f32) {
        self.event_handler
            .touch_event(&mut self.context, phase, id, x, y);
    }
}

/// Starts the game. Takes a start configuration, allowing you to specify additional options like
/// high-dpi behavior, as well as a function specifying how to create the event handler from the
/// new context.
pub fn start<F, E>(conf: conf::Conf, f: F) -> GameResult
where
    E: std::error::Error + 'static,
    F: 'static + FnOnce(&mut Context) -> Box<dyn event::EventHandler<E>>,
{
    let fs = Filesystem::new(&conf);
    let quad_conf = conf.into();

    miniquad::start(quad_conf, |ctx| {
        let mut context = Context::new(ctx, fs);

        // uncommenting this leads to wrong window sizes as `set_window_size` is currently buggy
        //context.quad_ctx.set_window_size(800 as u32, 600 as u32);
        let (d_w, d_h) = context.quad_ctx.screen_size();
        context
            .gfx_context
            .set_screen_coordinates(graphics::Rect::new(0., 0., d_w, d_h));

        let event_handler = f(&mut context);

        miniquad::UserData::free(EventHandlerWrapper {
            event_handler,
            context,
        })
    });
    Ok(())
}
