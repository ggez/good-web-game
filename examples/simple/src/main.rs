#[cfg(not(target_arch = "wasm32"))]
extern crate ggez;
#[cfg(target_arch = "wasm32")]
extern crate good_web_game as ggez;

use ggez::{event, graphics, Context, GameResult};

struct MainState {
    image: graphics::Image,
    x: f32,
    y: f32,
}

impl MainState {
    fn new(ctx: &mut Context) -> MainState {
        // graphics::set_screen_coordinates(ctx, graphics::Rect::new(-1., -1., 1., 1.)).unwrap();

        MainState {
            image: graphics::Image::new(ctx, "/rust-logo-64x64-blk.png").unwrap(),
            x: 0.,
            y: 0.,
        }
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _: &mut Context) -> GameResult {
        Ok(())
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        // let rect = graphics::screen_coordinates(ctx);
        // let size = graphics::size(ctx);
        self.x = x; //rect.x + x * rect.w / size.0 as f32;
        self.y = y; //rect.y + y * rect.h / size.1 as f32;
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        graphics::draw(
            ctx,
            &self.image,
            graphics::DrawParam::default()
                .dest([self.x, self.y])
                .scale([1., 1.]),
        )
        .unwrap();
        graphics::present(ctx)
    }
}

#[cfg(target_arch = "wasm32")]
fn main() -> GameResult {
    use ggez::conf;

    good_web_game::start(
        conf::Conf {
            cache: conf::Cache::List(vec!["/rust-logo-64x64-blk.png"]),
            ..Default::default()
        },
        |mut context| {
            let state = MainState::new(&mut context);
            event::run(context, state)
        },
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> GameResult {
    let (mut context, mut event) = ggez::ContextBuilder::new("Image", "GoodWebGame")
        .add_resource_path("./static")
        .build()
        .unwrap();
    let mut state = MainState::new(&mut context);

    event::run(&mut context, &mut event, &mut state)
}
