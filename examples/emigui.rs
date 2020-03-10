extern crate cgmath;
extern crate emigui;
extern crate emigui_miniquad;
extern crate good_web_game as ggez;
extern crate nalgebra;

use ggez::event;
use ggez::graphics;
use ggez::{Context, GameResult};

use std::f32;

use graphics::{Point2, Rect};
use {
    emigui::{
        label,
        math::vec2,
        widgets::{Button, Label},
        Align, Emigui,
    },
    emigui_miniquad::Painter,
};

struct App {
    emigui: Emigui,
    raw_input: emigui::RawInput,
    painter: Painter,
}

impl App {
    fn new(ctx: &mut Context) -> GameResult<App> {
        let pixels_per_point = ctx.quad_ctx.dpi_scale();
        let raw_input = emigui::RawInput {
            screen_size: {
                let (width, height) = ctx.quad_ctx.screen_size();
                vec2(width as f32, height as f32) / pixels_per_point
            },
            pixels_per_point,
            ..Default::default()
        };

        Ok(App {
            emigui: Emigui::new(pixels_per_point),
            painter: Painter::new(&mut ctx.quad_ctx),
            raw_input,
        })
    }
}

impl event::EventHandler for App {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        graphics::set_screen_coordinates(ctx, graphics::Rect::new(0.0, 0.0, width, height))
            .unwrap();

        self.raw_input.screen_size = vec2(width, height);
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _: ggez::input::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.raw_input.mouse_down = true;
    }
    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        _: ggez::input::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.raw_input.mouse_down = false;
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.raw_input.mouse_pos = Some(vec2(x as f32, y as f32));
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        // here is an example of emigui usage
        // NOTE: it use raw miniquad handle, gwg's settings like coordinate_system are ignored
        {
            self.emigui.new_frame(self.raw_input);
            let mut region = self.emigui.whole_screen_region();
            let mut region = region.left_column(region.width().min(480.0));
            region.set_align(Align::Min);
            region.add(
                label!("Emigui running inside of Miniquad").text_style(emigui::TextStyle::Heading),
            );
            if region.add(Button::new("Quit")).clicked {
                // cant quit the web
            }
            self.emigui.example(&mut region);
            let mesh = self.emigui.paint();
            let texture = self.emigui.texture();

            self.painter.paint(&mut ctx.quad_ctx, mesh, texture);
        }

        // drawing with gwg's "graphics"
        {
            let font_polygon = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect::new(0., 0., 100., 100.),
                graphics::Color::from_rgba(0, 255, 0, 255),
            )?;

            graphics::draw(
                ctx,
                &font_polygon,
                (
                    Point2::new(500.0, 200.0),
                    graphics::Color::new(1., 1., 1., 1.),
                ),
            )?;
        }

        graphics::present(ctx)?;

        Ok(())
    }
}

pub fn main() -> GameResult {
    ggez::start(
        ggez::conf::Conf {
            cache: ggez::conf::Cache::Tar(include_bytes!("resources.tar").to_vec()),
            loading: ggez::conf::Loading::Embedded,
            ..Default::default()
        },
        |mut context| Box::new(App::new(&mut context).unwrap()),
    )
}
