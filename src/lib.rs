#![allow(warnings)]

pub mod conf;
pub mod error;
pub mod event;
pub mod filesystem;
pub mod goodies;
pub mod graphics;
pub mod input;
pub mod timer;

mod context;

pub use crate::{
    context::{Context, ContextInternal},
    error::GameError,
    error::GameResult,
    event::EventHandler,
    goodies::matrix_transform_2d,
};
pub use cgmath;

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

struct EventHandlderWrapper {
    event_handler: Box<event::EventHandler>,
    context_internal: ContextInternal,
}

impl miniquad::EventHandler for EventHandlderWrapper {
    fn update(&mut self, ctx: &mut miniquad::Context) {
        self.event_handler.update(&mut Context {
            internal: &mut self.context_internal,
            quad_ctx: ctx,
        });
        self.context_internal.timer_context.tick();
    }

    fn draw(&mut self, ctx: &mut miniquad::Context) {
        self.event_handler.draw(&mut Context {
            internal: &mut self.context_internal,
            quad_ctx: ctx,
        });
    }

    fn resize_event(&mut self, ctx: &mut miniquad::Context, width: f32, height: f32) {
        self.event_handler.resize_event(
            &mut Context {
                internal: &mut self.context_internal,
                quad_ctx: ctx,
            },
            width,
            height,
        );
    }

    fn key_down_event(
        &mut self,
        ctx: &mut miniquad::Context,
        keycode: miniquad::KeyCode,
        keymods: miniquad::KeyMods,
        repeat: bool,
    ) {
        self.event_handler.key_down_event(
            &mut Context {
                internal: &mut self.context_internal,
                quad_ctx: ctx,
            },
            keycode.into(),
            crate::input::keyboard::KeyMods::NONE,
            repeat,
        );
    }

    fn key_up_event(
        &mut self,
        ctx: &mut miniquad::Context,
        keycode: miniquad::KeyCode,
        _keymods: miniquad::KeyMods,
    ) {
        self.event_handler.key_up_event(
            &mut Context {
                internal: &mut self.context_internal,
                quad_ctx: ctx,
            },
            keycode.into(),
            crate::input::keyboard::KeyMods::NONE,
        );
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut miniquad::Context,
        button: miniquad::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.event_handler.mouse_button_up_event(
            &mut Context {
                internal: &mut self.context_internal,
                quad_ctx: ctx,
            },
            button.into(),
            x,
            y,
        );
    }

    fn mouse_motion_event(
        &mut self,
        ctx: &mut miniquad::Context,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
    ) {
        self.event_handler.mouse_motion_event(
            &mut Context {
                internal: &mut self.context_internal,
                quad_ctx: ctx,
            },
            x,
            y,
            dx,
            dy,
        );
    }
}

pub fn start<F>(conf: conf::Conf, f: F) -> GameResult
where
    F: 'static + FnOnce(&mut Context) -> Box<dyn EventHandler>,
{
    miniquad::start(miniquad::conf::Conf::default(), |ctx| {
        let mut context_internal = ContextInternal::new(conf);

        let (w, h) = ctx.screen_size();
        context_internal
            .gfx_context
            .set_screen_coordinates(graphics::Rect::new(0., 0., w as f32, h as f32));

        let event_handler = f(&mut Context {
            internal: &mut context_internal,
            quad_ctx: ctx,
        });

        Box::new(EventHandlderWrapper {
            event_handler: event_handler,
            context_internal,
        })
    });
    Ok(())
}
