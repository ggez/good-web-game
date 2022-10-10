//! This has been created as a test for the mouse input functions, but it's simple enough, so why not

extern crate good_web_game as ggez;

use ggez::input::mouse::set_cursor_grabbed;
use ggez::{event::EventHandler, miniquad, Context, GameResult};

fn main() -> GameResult<()> {
    ggez::start(ggez::conf::Conf::default(), |_context, _quad_ctx| {
        Box::new(MousePos(0., 0.))
    })
}

struct MousePos(f32, f32);

impl EventHandler for MousePos {
    fn update(
        &mut self,
        ctx: &mut Context,
        quad_ctx: &mut miniquad::graphics::GraphicsContext,
    ) -> GameResult<()> {
        set_cursor_grabbed(ctx, quad_ctx, true);
        println!("UPDATE: delta: {:?}", ggez::input::mouse::delta(ctx));
        Ok(())
    }

    fn draw(
        &mut self,
        ctx: &mut Context,
        quad_ctx: &mut miniquad::graphics::GraphicsContext,
    ) -> GameResult<()> {
        ggez::graphics::present(ctx, quad_ctx)
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        _quad_ctx: &mut miniquad::graphics::GraphicsContext,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
    ) {
        let delta = (x - self.0, y - self.1);
        *self = MousePos(x, y);
        println!(
            "{delta:6?} == ({dx:6?}, {dy:6?}) = {eq}",
            delta = delta,
            dx = dx,
            dy = dy,
            eq = delta == (dx, dy)
        );
    }
}
