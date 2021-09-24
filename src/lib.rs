//! # Good Web Game
//!
//! [![Discord chat](https://img.shields.io/discord/710177966440579103.svg?label=discord%20chat)](https://discord.gg/jum3Fjek2A)
//!
//! good-web-game is a wasm32-unknown-unknown implementation of a [ggez](https://github.com/ggez/ggez) subset on top of [miniquad](https://github.com/not-fl3/miniquad/). Originally built to run [Zemeroth](https://github.com/ozkriff/zemeroth) on the web.
//!
//! It has been recently updated to support much of the ggez 0.6.1 API. If you're already working with ggez you might use this library to port your game to the web (or perhaps even mobile).
//! Since it also runs well on desktop it also offers an alternative implementation of ggez, which might always come in handy.
//!
//! If you are just looking for a well supported minimal high-level engine on top of miniquad you might want to take a look at [macroquad](https://github.com/not-fl3/macroquad/).
//!
//! ## Status
//!
//! "good-web-game" implements most of the ggez 0.6.1 API.
//!
//! ### Differences
//!
//! * boilerplate code differs slightly, [as shown here](https://github.com/PSteinhaus/PSteinhaus.github.io/tree/main/ggez/web-examples#ggez-animation-example)
//! * shaders have to be written in GLSL100, due to support for WebGL1
//! * API for creation of shaders and their corresponding uniform structs differs slightly, but the workflow remains the same, see [the `shader` example](examples/shader.rs)
//!
//! ### Missing / Not available:
//!
//! * filesystem with writing access (if you need it take a look at [`quad-storage`](https://github.com/optozorax/quad-storage))
//! * writing your own event loop (doesn't make much sense on callback-only platforms like HTML5)
//! * spatial audio (overall audio support is still relatively limited, but could be improved)
//! * resolution control in fullscreen mode
//! * setting window position / size (the latter is available on Windows, but buggy)
//! * screenshot function
//! * window icon
//! * gamepad support on WASM (as `gilrs` depends on wasm-bindgen)
//!
//! ### On blurry graphics
//!
//! You may run into somewhat blurry graphics. This is caused by high-dpi rendering:
//!
//! When run on a system with a scaling factor unequal to 1 the graphics may appear blurry, due to the drawbuffer being scaled up, to achieve a window of the size requested by your OS.
//! This size is usually "the size you specified in `Conf`" * "your OS scaling factor".
//!
//! To avoid this set `Conf::high_dpi` to `true`. This leads to the drawbuffer being the size of your actual physical window. It also means though that you can't be sure how big your drawable space will actually be, as this will then depend on where the program is being run.
//!
//! We aim towards changing this, so that windows are always created with the physical size specified in `Conf`, but that's not directly supported by miniquad currently.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/ggez/good-web-game/master/about/logo.png"
)]

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
