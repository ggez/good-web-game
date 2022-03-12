//! This has been created as a test for the mouse input functions, but it's simple enough, so why not

extern crate good_web_game as ggez;

use ggez::input::mouse::set_cursor_grabbed;
use ggez::{event::EventHandler, Context, GameError, GameResult};

fn main() -> GameResult<()> {
    ggez::start(
        ggez::conf::Conf::default(),
        |context| {
            set_cursor_grabbed(context, true);
            Box::new(MousePos(0., 0.))
        },
    )
}

struct MousePos(f32, f32);

impl EventHandler<GameError> for MousePos {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        println!("UPDATE: delta: {:?}", ggez::input::mouse::delta(ctx));
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        ggez::graphics::present(ctx)
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) {
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
