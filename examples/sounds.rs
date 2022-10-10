extern crate glam;
extern crate good_web_game as ggez;

use ggez::audio;
use ggez::event;
use ggez::graphics;
use ggez::input;
use ggez::{Context, GameResult};

use ggez::event::quit;
use quad_snd::PlaySoundParams;

struct MainState {
    sound: audio::Source,
    volume: f32,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let sound = audio::Source::new(ctx, "/sound.ogg")?;
        let s = MainState {
            sound,
            volume: PlaySoundParams::default().volume,
        };
        Ok(s)
    }

    fn play_once(&mut self, ctx: &mut Context) -> GameResult {
        self.sound.set_repeat(false);
        self.sound.play(ctx)
    }

    fn play_repeating(&mut self, ctx: &mut Context) -> GameResult {
        self.sound.set_repeat(true);
        self.sound.play(ctx)
    }

    fn stop(&self, ctx: &mut Context) -> GameResult {
        self.sound.stop(ctx)
    }

    fn increase_volume(&mut self, ctx: &mut Context) -> GameResult {
        self.volume += 0.1;
        self.sound.set_volume(ctx, self.volume)
    }

    fn decrease_volume(&mut self, ctx: &mut Context) -> GameResult {
        self.volume -= 0.1;
        self.sound.set_volume(ctx, self.volume)
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(
        &mut self,
        _ctx: &mut Context,
        _quad_ctx: &mut miniquad::GraphicsContext,
    ) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, quad_ctx: &mut miniquad::GraphicsContext) -> GameResult {
        graphics::clear(ctx, quad_ctx, [0.1, 0.2, 0.3, 1.0].into());

        let text = graphics::Text::new("Press number key 1 to to play a sound,\n2 to play repeated,\n3 to stop,\nUp to increase volume,\nDown to decrease volume,\nor escape to quit.");
        graphics::draw(
            ctx,
            quad_ctx,
            &text,
            (glam::Vec2::new(100.0, 100.), graphics::Color::WHITE),
        )?;

        graphics::present(ctx, quad_ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        _quad_ctx: &mut miniquad::GraphicsContext,
        keycode: event::KeyCode,
        _keymod: input::keyboard::KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            event::KeyCode::Key1 => self.play_once(ctx).unwrap(),
            event::KeyCode::Key2 => self.play_repeating(ctx).unwrap(),
            event::KeyCode::Key3 => self.stop(ctx).unwrap(),
            event::KeyCode::Up => self.increase_volume(ctx).unwrap(),
            event::KeyCode::Down => self.decrease_volume(ctx).unwrap(),
            event::KeyCode::Escape => {
                quit(ctx);
            }
            _ => (),
        }
    }
}

pub fn main() -> GameResult {
    ggez::start(
        ggez::conf::Conf::default().cache(Some(include_bytes!("resources.tar"))),
        |ctx, _quad_ctx| Box::new(MainState::new(ctx).unwrap()),
    )
}
