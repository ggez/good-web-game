/// Highly experimental and not ggez-compatible things

use good_web_game::{
    cgmath::{Point2, Vector2},
    event,
    goodies::megaui::Ui,
    graphics, hash,
    input::MouseButton,
    Context, GameResult,
};

struct MainState {
    ui: Ui,
    counter: u32,
}

impl MainState {
    fn new(_: &mut Context) -> MainState {
        MainState {
            ui: Ui::new(),
            counter: 0,
        }
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _: &mut Context) -> GameResult {
        Ok(())
    }

    fn mouse_motion_event(&mut self, _: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.ui.mouse_move(Point2::new(x, y));
    }

    fn mouse_button_down_event(&mut self, _: &mut Context, _: MouseButton, x: f32, y: f32) {
        self.ui.mouse_down(Point2::new(x, y));
    }

    fn mouse_button_up_event(&mut self, _: &mut Context, _: MouseButton, x: f32, y: f32) {
        self.ui.mouse_up(Point2::new(x, y));
    }

    fn mouse_wheel_event(&mut self, _: &mut Context, x: f32, y: f32) {
        self.ui.mouse_wheel(x, y);
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [1., 1., 1., 1.0].into());

        self.ui.begin_frame();

        let counter = &mut self.counter;
        self.ui.window(
            hash!(),
            "Hello!",
            Point2::new(50., 50.),
            Vector2::new(200., 100.),
            |ui| {
                ui.label(Point2::new(20., 20.), &format!("Counter: {}", counter));
                if ui.button(Point2::new(100., 50.), hash!(), "increment") {
                    *counter += 1;
                }
            },
        );
        self.ui.draw(ctx);
        graphics::present(ctx)
    }
}

fn main() -> GameResult {
    use good_web_game::conf;

    good_web_game::start(
        conf::Conf {
            cache: conf::Cache::No,
            ..Default::default()
        },
        |mut context| {
            let state = MainState::new(&mut context);
            event::run(context, state)
        },
    )
}
