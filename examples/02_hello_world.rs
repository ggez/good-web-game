//! Basic hello world example.
extern crate good_web_game as ggez;

use ggez::event;
use ggez::graphics;
use ggez::{Context, GameResult};
use std::env;
use std::path;

// First we make a structure to contain the game's state
struct MainState {
    frames: usize,
    text: graphics::Text,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        // The ttf file will be in your resources directory. Later, we
        // will mount that directory so we can omit it in the path here.
        let font = graphics::Font::new(ctx, "/LiberationMono-Regular.ttf")?;
        let text = graphics::Text::new(("Hello world!", font, 48.0));

        let s = MainState { frames: 0, text };
        Ok(s)
    }
}

// Then we implement the `ggez:event::EventHandler` trait on it, which
// requires callbacks for updating and drawing the game state each frame.
//
// The `EventHandler` trait also contains callbacks for event handling
// that you can override if you wish, but the defaults are fine.
impl event::EventHandler for MainState {
    fn update(
        &mut self,
        _ctx: &mut Context,
        _quad_ctx: &mut miniquad::graphics::GraphicsContext,
    ) -> GameResult {
        Ok(())
    }

    fn draw(
        &mut self,
        ctx: &mut Context,
        quad_ctx: &mut miniquad::graphics::GraphicsContext,
    ) -> GameResult {
        graphics::clear(ctx, quad_ctx, [0.1, 0.2, 0.3, 1.0].into());

        // Drawables are drawn from their top-left corner.
        let offset = self.frames as f32 / 10.0;
        let dest_point = cgmath::Point2::new(offset, offset);
        graphics::draw(ctx, quad_ctx, &self.text, (dest_point,))?;
        graphics::present(ctx, quad_ctx)?;

        self.frames += 1;
        if (self.frames % 100) == 0 {
            println!("FPS: {}", ggez::timer::fps(ctx));
            println!("drawable size: {:?}", graphics::drawable_size(quad_ctx));
        }

        Ok(())
    }
}

// Now our main function:
pub fn main() -> GameResult {
    // We add the CARGO_MANIFEST_DIR/resources to the resource paths
    // so that ggez will look in our cargo project directory for files.
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    ggez::start(
        ggez::conf::Conf::default()
            .cache(Some(include_bytes!("resources.tar")))
            .physical_root_dir(Some(resource_dir)),
        |mut context, mut _quad_ctx| Box::new(MainState::new(&mut context).unwrap()),
    )
}
