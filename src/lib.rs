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
use crate::input::mouse;
#[cfg(feature = "log-impl")]
pub use miniquad::{debug, info, log, warn};

pub mod rand {
    use miniquad::rand;

    pub trait RandomRange {
        fn gen_range(low: Self, high: Self) -> Self;
    }

    impl RandomRange for f32 {
        fn gen_range(low: Self, high: Self) -> Self {
            let r = unsafe { rand() } as f32 / miniquad::RAND_MAX as f32;
            low + (high - low) * r
        }
    }
    impl RandomRange for i32 {
        fn gen_range(low: i32, high: i32) -> Self {
            let r = unsafe { rand() } as f32 / miniquad::RAND_MAX as f32;
            let r = low as f32 + (high as f32 - low as f32) * r;
            r as i32
        }
    }
    impl RandomRange for i16 {
        fn gen_range(low: i16, high: i16) -> Self {
            let r = unsafe { rand() } as f32 / miniquad::RAND_MAX as f32;
            let r = low as f32 + (high as f32 - low as f32) * r;
            r as i16
        }
    }

    impl RandomRange for usize {
        fn gen_range(low: usize, high: usize) -> Self {
            let r = unsafe { rand() } as f32 / miniquad::RAND_MAX as f32;
            let r = low as f32 + (high as f32 - low as f32) * r;
            r as usize
        }
    }

    pub fn gen_range<T>(low: T, high: T) -> T
    where
        T: RandomRange,
    {
        T::gen_range(low, high)
    }

    pub trait ChooseRandom<T> {
        fn choose(&mut self) -> Option<&mut T>;
    }

    impl<T> ChooseRandom<T> for Vec<T> {
        fn choose(&mut self) -> Option<&mut T> {
            let ix = gen_range(0, self.len());
            self.get_mut(ix)
        }
    }
}

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

    fn char_event(&mut self, character: char, _keymods: miniquad::KeyMods, _repeat: bool) {
        self.event_handler
            .text_input_event(&mut self.context, character);
    }
}

pub fn start<F, E>(conf: conf::Conf, f: F) -> GameResult
where
    E: std::error::Error + 'static,
    F: 'static + FnOnce(&mut Context) -> Box<dyn EventHandler<E>>,
{
    miniquad::start(miniquad::conf::Conf::default(), |ctx| {
        let mut context = Context::new(ctx, conf);

        let (w, h) = context.quad_ctx.screen_size();
        context
            .gfx_context
            .set_screen_coordinates(graphics::Rect::new(0., 0., w as f32, h as f32));

        let event_handler = f(&mut context);

        miniquad::UserData::free(EventHandlerWrapper {
            event_handler,
            context,
        })
    });
    Ok(())
}
