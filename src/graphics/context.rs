use crate::{
    graphics::{types::Rect, Canvas},
    GameResult,
};
use miniquad_text_rusttype::{FontAtlas, FontTexture};
use std::rc::Rc;

use cgmath::Matrix4;

const DEFAULT_FONT_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/DejaVuSerif.ttf"
));

pub struct GraphicsContext {
    pub(crate) screen_rect: Rect,
    pub(crate) projection: Matrix4<f32>,
    pub(crate) white_texture: miniquad::Texture,
    //pub(crate) text_cache: HashMap<String, GpuText>,
    pub(crate) canvas: Option<Canvas>,
    pub(crate) sprite_pipeline: miniquad::Pipeline,
    pub(crate) mesh_pipeline: miniquad::Pipeline,
    pub(crate) image_pipeline: miniquad::Pipeline,
    pub(crate) text_system: miniquad_text_rusttype::TextSystem,
    pub(crate) fonts_cache: Vec<Rc<miniquad_text_rusttype::FontTexture>>,
    pub(crate) font_size: u32,
}

impl GraphicsContext {
    pub fn new(ctx: &mut miniquad::Context) -> GraphicsContext {
        use miniquad::*;

        let projection = cgmath::One::one();
        let screen_rect = Rect::new(-1., -1., 2., 2.);

        let white_texture = Texture::from_rgba8(ctx, 1, 1, &[255, 255, 255, 255]);

        let sprite_shader = Shader::new(
            ctx,
            batch_shader::VERTEX,
            batch_shader::FRAGMENT,
            batch_shader::meta(),
        )
        .expect("couldn't create sprite shader");

        let sprite_pipeline = miniquad::Pipeline::with_params(
            ctx,
            &[
                BufferLayout::default(),
                BufferLayout {
                    step_func: VertexStep::PerInstance,
                    ..Default::default()
                },
            ],
            &[
                VertexAttribute::with_buffer("position", VertexFormat::Float2, 0),
                VertexAttribute::with_buffer("Source", VertexFormat::Float4, 1),
                VertexAttribute::with_buffer("Color", VertexFormat::Float4, 1),
                VertexAttribute::with_buffer("InstanceModel", VertexFormat::Mat4, 1),
            ],
            sprite_shader,
            PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
        );

        let image_shader = Shader::new(
            ctx,
            image_shader::VERTEX,
            image_shader::FRAGMENT,
            image_shader::meta(),
        )
        .expect("couldn't create image shader");

        let image_pipeline = miniquad::Pipeline::with_params(
            ctx,
            &[BufferLayout::default()],
            &[VertexAttribute::with_buffer(
                "position",
                VertexFormat::Float2,
                0,
            )],
            image_shader,
            PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
        );

        let mesh_shader = Shader::new(
            ctx,
            mesh_shader::VERTEX,
            mesh_shader::FRAGMENT,
            mesh_shader::meta(),
        )
        .expect("couldn't create mesh shader");

        let mesh_pipeline = Pipeline::with_params(
            ctx,
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("position", VertexFormat::Float2),
                VertexAttribute::new("texcoord", VertexFormat::Float2),
                VertexAttribute::new("color0", VertexFormat::Float4),
            ],
            mesh_shader,
            PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
        );

        let text_system = miniquad_text_rusttype::TextSystem::new(ctx);

        // load default font, will be available by FontId::default()
        let fonts_cache = vec![Rc::new(load_font(ctx, DEFAULT_FONT_BYTES, 70).unwrap())];

        GraphicsContext {
            projection,
            screen_rect,
            white_texture,
            canvas: None,
            sprite_pipeline,
            mesh_pipeline,
            image_pipeline,
            text_system,
            fonts_cache,
            font_size: 50,
        }
    }
}

impl GraphicsContext {
    pub(crate) fn load_font(
        &mut self,
        ctx: &mut miniquad::Context,
        font_bytes: &[u8],
        font_size: u32,
    ) -> GameResult<usize> {
        let font = load_font(ctx, &font_bytes, font_size)?;

        self.fonts_cache.push(Rc::new(font));

        Ok(self.fonts_cache.len() - 1)
    }

    /// Sets the raw projection matrix to the given Matrix.
    ///
    /// Call `update_globals()` to apply after calling this.
    pub(crate) fn set_projection(&mut self, mat: Matrix4<f32>) {
        self.projection = mat;
    }

    /// Gets a copy of the raw projection matrix.
    pub(crate) fn projection(&self) -> Matrix4<f32> {
        self.projection
    }

    pub fn set_screen_coordinates(&mut self, rect: crate::graphics::types::Rect) {
        self.screen_rect = rect;
        self.projection =
            cgmath::ortho(rect.x, rect.x + rect.w, rect.y + rect.h, rect.y, -1.0, 1.0);
    }
}

fn load_font(
    ctx: &mut miniquad::Context,
    font_data: &[u8],
    font_size: u32,
) -> GameResult<FontTexture> {
    Ok(FontTexture::new(
        ctx,
        font_data,
        font_size,
        FontAtlas::ascii_character_list(), // TODO: check whether `FontAtlas::ascii_character_list()`
                                           //      is a proper drop in replacement for `FontTexture::ascii_character_list()`
    )?)
}

pub(crate) mod batch_shader {
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformDesc, UniformType};

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 position;
    attribute vec4 Source;
    attribute vec4 Color;
    attribute mat4 InstanceModel;

    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform mat4 Projection;
    uniform mat4 Model;

    uniform float depth;

    void main() {
        gl_Position = Projection * Model * InstanceModel * vec4(position, 0, 1);
        gl_Position.z = depth;
        color = Color;
        uv = position * Source.zw + Source.xy;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform sampler2D Texture;

    void main() {
        gl_FragColor = texture2D(Texture, uv) * color;
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["Texture".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![
                    UniformDesc::new("Projection", UniformType::Mat4),
                    UniformDesc::new("Model", UniformType::Mat4),
                ],
            },
        }
    }

    #[repr(C)]
    #[derive(Debug)]
    pub struct Uniforms {
        pub projection: cgmath::Matrix4<f32>,
        pub model: cgmath::Matrix4<f32>,
    }
}

pub(crate) mod image_shader {
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformDesc, UniformType};

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 position;

    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform mat4 Projection;
    uniform vec4 Source;
    uniform vec4 Color;
    uniform mat4 Model;

    uniform float depth;

    void main() {
        gl_Position = Projection * Model * vec4(position, 0, 1);
        gl_Position.z = depth;
        color = Color;
        uv = position * Source.zw + Source.xy;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform sampler2D Texture;

    void main() {
        gl_FragColor = texture2D(Texture, uv) * color;
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["Texture".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![
                    UniformDesc::new("Projection", UniformType::Mat4),
                    UniformDesc::new("Source", UniformType::Float4),
                    UniformDesc::new("Color", UniformType::Float4),
                    UniformDesc::new("Model", UniformType::Mat4),
                ],
            },
        }
    }

    #[repr(C)]
    #[derive(Debug)]
    pub struct Uniforms {
        pub projection: cgmath::Matrix4<f32>,
        pub source: cgmath::Vector4<f32>,
        pub color: cgmath::Vector4<f32>,
        pub model: cgmath::Matrix4<f32>,
    }
}

pub(crate) mod mesh_shader {
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformDesc, UniformType};

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 position;
    attribute vec2 texcoord;
    attribute vec4 color0;

    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform mat4 Projection;
    uniform mat4 Model;
    uniform vec4 Color;

    uniform float depth;

    void main() {
        gl_Position = Projection * Model * vec4(position, 0, 1);
        gl_Position.z = depth;
        color = Color * color0;
        uv = texcoord;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform sampler2D Texture;

    void main() {
        gl_FragColor = texture2D(Texture, uv) * color;
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["Texture".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![
                    UniformDesc::new("Projection", UniformType::Mat4),
                    UniformDesc::new("Model", UniformType::Mat4),
                    UniformDesc::new("Color", UniformType::Float4),
                ],
            },
        }
    }

    #[repr(C)]
    #[derive(Debug)]
    pub struct Uniforms {
        pub projection: cgmath::Matrix4<f32>,
        pub model: cgmath::Matrix4<f32>,
        pub color: cgmath::Vector4<f32>,
    }
}
