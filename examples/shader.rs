//! A very simple shader example.

extern crate good_web_game as ggez;

use ggez::event;
use ggez::graphics::{
    self, Color, DrawMode, ShaderMeta, UniformBlockLayout, UniformDesc, UniformType,
};
use ggez::timer;
use ggez::{Context, GameResult};
//use std::env;
//use std::path;

// Define the input struct for our shader.
fn shader_meta() -> ShaderMeta {
    ShaderMeta {
        images: vec!["Texture".to_string()],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("u_Rate", UniformType::Float1),
                // `Projection` always comes last, as it is appended by gwg internally
                UniformDesc::new("Projection", UniformType::Mat4),
            ],
        },
    }
}

use bytemuck_derive::{Pod, Zeroable};
#[repr(C)]
#[derive(Copy, Clone, Zeroable, Pod)]
// The attributes above ensure that this struct derives bytemuck::Pod,
// which is necessary for it to be passed to `graphics::set_uniforms`
pub struct ExtraUniforms {
    pub u_rate: f32,
}

struct MainState {
    shader_id: graphics::ShaderId,
    dim: f32,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let dim = 0.5;
        let shader_id = graphics::Shader::new(
            ctx,
            "/basic_150.glslv",
            "/dimmer_150.glslf",
            shader_meta(),
            None,
        )?;
        Ok(MainState { shader_id, dim })
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.dim = 0.5 + (((timer::ticks(ctx) as f32) / 100.0).cos() / 2.0);
        graphics::set_uniforms(ctx, self.shader_id, ExtraUniforms { u_rate: self.dim });
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let circle = graphics::Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            glam::Vec2::new(100.0, 300.0),
            100.0,
            2.0,
            Color::WHITE,
        )?;
        graphics::draw(ctx, &circle, (glam::Vec2::new(0.0, 0.0),))?;

        {
            let _lock = graphics::use_shader(ctx, self.shader_id);
            let circle = graphics::Mesh::new_circle(
                ctx,
                DrawMode::fill(),
                glam::Vec2::new(400.0, 300.0),
                100.0,
                2.0,
                Color::WHITE,
            )?;
            graphics::draw(ctx, &circle, (glam::Vec2::new(0.0, 0.0),))?;
        }

        let circle = graphics::Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            glam::Vec2::new(700.0, 300.0),
            100.0,
            2.0,
            Color::WHITE,
        )?;
        graphics::draw(ctx, &circle, (glam::Vec2::new(0.0, 0.0),))?;

        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn main() -> GameResult {
    /*
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };
    */
    ggez::start(
        ggez::conf::Conf::default()
            .cache(Some(include_bytes!("resources.tar"))),
        |mut context| Box::new(MainState::new(&mut context).unwrap()),
    )
}
