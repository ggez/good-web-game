//! https://github.com/ggez/ggez/blob/master/examples/03_drawing.rs
//! A collection of semi-random shape and image drawing examples.

extern crate glam;
extern crate good_web_game as ggez;

use ggez::event;
use ggez::graphics::{self, Canvas, Color, DrawMode, DrawParam, FilterMode};
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::timer;
use ggez::{Context, GameResult};
//use lyon::lyon_tessellation::FillOptions;
use ggez::miniquad;
use std::env;
use std::path;

type Point2 = glam::Vec2;

struct MainState {
    image1: graphics::Image,
    image2_linear: graphics::Image,
    image2_nearest: graphics::Image,
    meshes: Vec<graphics::Mesh>,
    zoomlevel: f32,
    canvas: Canvas,
    use_canvas: bool,
}

impl MainState {
    fn new(ctx: &mut Context, quad_ctx: &mut miniquad::GraphicsContext) -> GameResult<MainState> {
        let image1 = graphics::Image::new(ctx, quad_ctx, "dragon1.png")?;
        let image2_linear = graphics::Image::new(ctx, quad_ctx, "shot.png")?;
        let mut image2_nearest = graphics::Image::new(ctx, quad_ctx, "shot.png")?;
        image2_nearest.set_filter(graphics::FilterMode::Nearest);
        let canvas = Canvas::with_window_size(ctx, quad_ctx)?;

        let meshes = vec![
            build_mesh(ctx, quad_ctx)?,
            build_textured_triangle(ctx, quad_ctx)?,
        ];
        let s = MainState {
            image1,
            image2_linear,
            image2_nearest,
            meshes,
            zoomlevel: 1.0,
            canvas,
            use_canvas: false,
        };

        Ok(s)
    }
}

fn build_mesh(
    ctx: &mut Context,
    quad_ctx: &mut miniquad::GraphicsContext,
) -> GameResult<graphics::Mesh> {
    let mb = &mut graphics::MeshBuilder::new();

    mb.line(
        &[
            Point2::new(200.0, 200.0),
            Point2::new(400.0, 200.0),
            Point2::new(400.0, 400.0),
            Point2::new(200.0, 400.0),
            Point2::new(200.0, 300.0),
        ],
        4.0,
        Color::new(1.0, 0.0, 0.0, 1.0),
    )?;

    mb.ellipse(
        DrawMode::fill(),
        Point2::new(600.0, 200.0),
        50.0,
        120.0,
        1.0,
        Color::new(1.0, 1.0, 0.0, 1.0),
    )?;

    mb.circle(
        DrawMode::fill(),
        Point2::new(600.0, 380.0),
        40.0,
        1.0,
        Color::new(1.0, 0.0, 1.0, 1.0),
    )?;

    mb.build(ctx, quad_ctx)
}

fn build_textured_triangle(
    ctx: &mut Context,
    quad_ctx: &mut miniquad::GraphicsContext,
) -> GameResult<graphics::Mesh> {
    let mb = &mut graphics::MeshBuilder::new();
    let triangle_verts = vec![
        graphics::Vertex {
            pos: [100.0, 100.0],
            uv: [1.0, 1.0],
            color: [1.0, 0.0, 0.0, 1.0],
        },
        graphics::Vertex {
            pos: [0.0, 100.0],
            uv: [0.0, 1.0],
            color: [0.0, 1.0, 0.0, 1.0],
        },
        graphics::Vertex {
            pos: [0.0, 0.0],
            uv: [0.0, 0.0],
            color: [0.0, 0.0, 1.0, 1.0],
        },
    ];

    let triangle_indices = vec![0, 1, 2];

    let mut i = graphics::Image::new(ctx, quad_ctx, "rock.png")?;
    i.set_filter(FilterMode::Nearest);
    mb.raw(&triangle_verts, &triangle_indices, Some(i))?;
    mb.build(ctx, quad_ctx)
}

impl event::EventHandler for MainState {
    fn update(
        &mut self,
        ctx: &mut Context,
        _quad_ctx: &mut miniquad::GraphicsContext,
    ) -> GameResult {
        const DESIRED_FPS: u32 = 60;

        while timer::check_update_time(ctx, DESIRED_FPS) {
            self.zoomlevel += 0.01;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, quad_ctx: &mut miniquad::GraphicsContext) -> GameResult {
        if self.use_canvas {
            graphics::set_canvas(ctx, Some(&self.canvas));
        }

        graphics::clear(ctx, quad_ctx, [0.1, 0.2, 0.3, 1.0].into());
        // let src = graphics::Rect::new(0.25, 0.25, 0.5, 0.5);
        // let src = graphics::Rect::one();
        let dst = cgmath::Point2::new(20.0, 20.0);
        graphics::draw(ctx, quad_ctx, &self.image1, (dst,))?;

        let dst = cgmath::Point2::new(200.0, 100.0);
        let dst2 = cgmath::Point2::new(400.0, 400.0);
        let scale = cgmath::Vector2::new(10.0, 10.0);
        //let shear = graphics::Point::new(self.zoomlevel, self.zoomlevel);
        graphics::draw(
            ctx,
            quad_ctx,
            &self.image2_linear,
            graphics::DrawParam::new()
                // src: src,
                .dest(dst)
                .rotation(self.zoomlevel)
                // offset: Point2::new(-16.0, 0.0),
                .scale(scale)
                .color(graphics::Color::new(1.0, 1.0, 1.0, 1.0)), // shear: shear,
        )?;
        graphics::draw(
            ctx,
            quad_ctx,
            &self.image2_nearest,
            graphics::DrawParam::new()
                //.src(src)
                .dest(dst2)
                .rotation(self.zoomlevel)
                .offset(Point2::new(0.5, 0.5))
                .scale(scale), // shear: shear,
        )?;

        let rect = graphics::Rect::new(450.0, 450.0, 50.0, 50.0);
        let r1 = graphics::Mesh::new_rectangle(
            ctx,
            quad_ctx,
            graphics::DrawMode::fill(),
            rect,
            graphics::Color::WHITE,
        )?;
        graphics::draw(ctx, quad_ctx, &r1, DrawParam::default())?;

        let rect = graphics::Rect::new(450.0, 450.0, 50.0, 50.0);
        let r2 = graphics::Mesh::new_rectangle(
            ctx,
            quad_ctx,
            graphics::DrawMode::fill(),
            rect,
            graphics::Color::new(1.0, 0.0, 0.0, 1.0),
        )?;
        graphics::draw(ctx, quad_ctx, &r2, DrawParam::default())?;
        //graphics::rectangle(ctx, graphics::WHITE, graphics::DrawMode::fill(), rect)?;

        // let rect = graphics::Rect::new(450.0, 450.0, 50.0, 50.0);
        // graphics::rectangle(
        //     ctx,
        //     graphics::Color::new(1.0, 0.0, 0.0, 1.0),
        //     graphics::DrawMode::stroke(1.0),
        //     rect,
        // )?;

        for m in &self.meshes {
            graphics::draw(ctx, quad_ctx, m, DrawParam::new())?;
        }

        if self.use_canvas {
            graphics::set_canvas(ctx, None);
            graphics::draw(ctx, quad_ctx, &self.canvas, DrawParam::new())?;
        }

        graphics::present(ctx, quad_ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        _quad_ctx: &mut miniquad::GraphicsContext,
        _keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        self.use_canvas ^= true;
        println!("use_canvas: {}", self.use_canvas);
    }
}

pub fn main() -> GameResult {
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
        //.sample_count(16),
        |mut context, quad_ctx| Box::new(MainState::new(&mut context, quad_ctx).unwrap()),
    )
}
