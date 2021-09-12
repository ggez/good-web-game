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

pub use crate::error::*;
pub use crate::{
    context::Context, error::GameError, error::GameResult, event::EventHandler,
    goodies::matrix_transform_2d,
};
pub use cgmath;

use crate::event::ErrorOrigin;
use crate::filesystem::Filesystem;
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
        }

        // in ggez tick is called before update, so I moved this to the front
        self.context.timer_context.tick();

        // release all buffers that were kept alive for the previous frame
        graphics::release_dropped_bindings();

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
        self.event_handler
            .mouse_button_down_event(&mut self.context, button.into(), x, y);
    }

    fn mouse_button_up_event(&mut self, button: miniquad::MouseButton, x: f32, y: f32) {
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

pub fn start<F, E>(conf: conf::Conf, f: F) -> GameResult
where
    E: std::error::Error + 'static,
    F: 'static + FnOnce(&mut Context) -> Box<dyn EventHandler<E>>,
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
