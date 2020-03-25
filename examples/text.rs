//! https://github.com/ggez/ggez/blob/master/examples/text.rs
//!
//! This example demonstrates how to use `Text` to draw TrueType font texts efficiently.
//!
//! Powered by `miniquad_text_rusttype` crate.

extern crate cgmath;
extern crate good_web_game as ggez;
extern crate nalgebra;

use ggez::event;
use ggez::graphics::{self, Font, Text};
use ggez::{Context, GameResult};
use nalgebra::Point2;
use std::f32;

struct App {
    fancy_font: Font,
}

impl App {
    fn new(ctx: &mut Context) -> GameResult<App> {
        let fancy_font = Font::new(ctx, "Tangerine_Regular.ttf")?;

        Ok(App { fancy_font })
    }
}

impl event::EventHandler for App {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let strings = [("AAija", 86.), ("jjjjj", 100.), ("i", 50.)];
        let vertical_spacing = 150.;

        for (i, (string, scale)) in strings.iter().enumerate() {
            let text_y = 50. + (*scale + vertical_spacing) * i as f32;
            let text = Text::new((*string, self.fancy_font, *scale));
            let dimensions = text.dimensions(ctx);

            let font_bounding_rect =
                graphics::Rect::new(0., 0., dimensions.0 as f32, dimensions.1 as f32);
            // let desired_bounding_rect = graphics::Rect::new(0., 0., dimensions.0 as f32, *scale);

            let font_polygon = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                font_bounding_rect,
                graphics::Color::from_rgba(255, 0, 0, 255),
            )?;
            // let desired_polygon = graphics::Mesh::new_rectangle(
            //     ctx,
            //     graphics::DrawMode::fill(),
            //     desired_bounding_rect,
            //     graphics::BLACK,
            // )?;

            graphics::draw(
                ctx,
                &font_polygon,
                (
                    Point2::new(200.0, text_y),
                    graphics::Color::new(0., 1., 0., 1.),
                ),
            )?;
            // graphics::draw(
            //     ctx,
            //     &desired_polygon,
            //     (Point2::new(50.0, text_y), graphics::WHITE),
            // )?;

            graphics::draw(ctx, &text, (Point2::new(200.0, text_y), graphics::WHITE))?;
        }

        graphics::present(ctx)?;

        Ok(())
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        graphics::set_screen_coordinates(ctx, graphics::Rect::new(0.0, 0.0, width, height))
            .unwrap();
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
