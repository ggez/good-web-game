//! Example that just prints out all the input events.

extern crate glam;
extern crate good_web_game as ggez;

use ggez::event::{self, KeyCode, KeyMods, MouseButton};
use ggez::graphics::{self, Color, DrawMode};
use ggez::input;
use ggez::miniquad;

#[cfg(not(any(target_arch = "wasm32", target_os = "ios", target_os = "android",)))]
use ggez::input::gamepad::{
    gilrs::{Axis, Button},
    GamepadId,
};
use ggez::{Context, GameResult};
use glam::*;

struct MainState {
    pos_x: f32,
    pos_y: f32,
    mouse_down: bool,
}

impl MainState {
    fn new() -> MainState {
        MainState {
            pos_x: 100.0,
            pos_y: 100.0,
            mouse_down: false,
        }
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(
        &mut self,
        ctx: &mut Context,
        _quad_ctx: &mut miniquad::GraphicsContext,
    ) -> GameResult {
        if input::keyboard::is_key_pressed(ctx, KeyCode::A) {
            println!("The A key is pressed");
            if input::keyboard::is_mod_active(ctx, input::keyboard::KeyMods::SHIFT) {
                println!("The shift key is held too.");
            }
            println!(
                "Full list of pressed keys: {:?}",
                input::keyboard::pressed_keys(ctx)
            );
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, quad_ctx: &mut miniquad::GraphicsContext) -> GameResult {
        graphics::clear(ctx, quad_ctx, [0.1, 0.2, 0.3, 1.0].into());

        let rectangle = graphics::Mesh::new_rectangle(
            ctx,
            quad_ctx,
            DrawMode::fill(),
            graphics::Rect {
                x: self.pos_x,
                y: self.pos_y,
                w: 400.0,
                h: 300.0,
            },
            Color::WHITE,
        )?;
        graphics::draw(ctx, quad_ctx, &rectangle, (glam::Vec2::new(0.0, 0.0),))?;

        graphics::present(ctx, quad_ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _quad_ctx: &mut miniquad::GraphicsContext,
        button: MouseButton,
        x: f32,
        y: f32,
    ) {
        self.mouse_down = true;
        println!("Mouse button pressed: {:?}, x: {}, y: {}", button, x, y);
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        _quad_ctx: &mut miniquad::GraphicsContext,
        button: MouseButton,
        x: f32,
        y: f32,
    ) {
        self.mouse_down = false;
        println!("Mouse button released: {:?}, x: {}, y: {}", button, x, y);
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        _quad_ctx: &mut miniquad::GraphicsContext,
        x: f32,
        y: f32,
        xrel: f32,
        yrel: f32,
    ) {
        if self.mouse_down {
            // Mouse coordinates are PHYSICAL coordinates, but here we want logical coordinates.

            // If you simply use the initial coordinate system, then physical and logical
            // coordinates are identical.
            self.pos_x = x;
            self.pos_y = y;

            // If you change your screen coordinate system you need to calculate the
            // logical coordinates like this:
            /*
            let screen_rect = graphics::screen_coordinates(_ctx);
            let size = graphics::window(_ctx).inner_size();
            self.pos_x = (x / (size.width  as f32)) * screen_rect.w + screen_rect.x;
            self.pos_y = (y / (size.height as f32)) * screen_rect.h + screen_rect.y;
            */
        }
        println!(
            "Mouse motion, x: {}, y: {}, relative x: {}, relative y: {}",
            x, y, xrel, yrel
        );
    }

    fn mouse_wheel_event(
        &mut self,
        _ctx: &mut Context,
        _quad_ctx: &mut miniquad::GraphicsContext,
        x: f32,
        y: f32,
    ) {
        println!("Mousewheel event, x: {}, y: {}", x, y);
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        _quad_ctx: &mut miniquad::GraphicsContext,
        keycode: KeyCode,
        keymod: KeyMods,
        repeat: bool,
    ) {
        println!(
            "Key pressed: {:?}, modifier {:?}, repeat: {}",
            keycode, keymod, repeat
        );
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut Context,
        _quad_ctx: &mut miniquad::GraphicsContext,
        keycode: KeyCode,
        keymod: KeyMods,
    ) {
        println!("Key released: {:?}, modifier {:?}", keycode, keymod);
    }

    fn text_input_event(
        &mut self,
        _ctx: &mut Context,
        _quad_ctx: &mut miniquad::GraphicsContext,
        ch: char,
    ) {
        println!("Text input: {}", ch);
    }

    #[cfg(not(any(target_arch = "wasm32", target_os = "ios", target_os = "android",)))]
    fn gamepad_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _quad_ctx: &mut miniquad::GraphicsContext,
        btn: Button,
        id: GamepadId,
    ) {
        println!("Gamepad button pressed: {:?} Gamepad_Id: {:?}", btn, id);
    }

    #[cfg(not(any(target_arch = "wasm32", target_os = "ios", target_os = "android",)))]
    fn gamepad_button_up_event(
        &mut self,
        _ctx: &mut Context,
        _quad_ctx: &mut miniquad::GraphicsContext,
        btn: Button,
        id: GamepadId,
    ) {
        println!("Gamepad button released: {:?} Gamepad_Id: {:?}", btn, id);
    }

    #[cfg(not(any(target_arch = "wasm32", target_os = "ios", target_os = "android",)))]
    fn gamepad_axis_event(
        &mut self,
        _ctx: &mut Context,
        _quad_ctx: &mut miniquad::GraphicsContext,
        axis: Axis,
        value: f32,
        id: GamepadId,
    ) {
        println!(
            "Axis Event: {:?} Value: {} Gamepad_Id: {:?}",
            axis, value, id
        );
    }

    /*
    fn focus_event(&mut self, _ctx: &mut Context, gained: bool) {
        if gained {
            println!("Focus gained");
        } else {
            println!("Focus lost");
        }
    }
    */
}

pub fn main() -> GameResult {
    //let cb = ggez::ContextBuilder::new("input_test", "ggez").window_mode(
    //    conf::WindowMode::default()
    //        .fullscreen_type(conf::FullscreenType::Windowed)
    //        .resizable(true),
    //);

    // remove the comment to see how physical mouse coordinates can differ
    // from logical game coordinates when the screen coordinate system changes
    // graphics::set_screen_coordinates(&mut ctx, Rect::new(20., 50., 2000., 1000.));

    // alternatively, resizing the window also leads to screen coordinates
    // and physical window size being out of sync

    ggez::start(
        ggez::conf::Conf::default()
            .cache(Some(include_bytes!("resources.tar")))
            .window_resizable(true),
        |_context, _quad_ctx| Box::new(MainState::new()),
    )
}
