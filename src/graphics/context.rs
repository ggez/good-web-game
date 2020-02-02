use crate::graphics::{spritebatch, text::Font, types::Rect, Canvas, DrawParam, Image};

use cgmath::{Matrix3, Matrix4};
use glyph_brush::{GlyphBrush, GlyphBrushBuilder};

use std::cell::RefCell;
use std::rc::Rc;

pub struct GraphicsContext {
    pub(crate) screen_rect: Rect,
    pub(crate) projection: Matrix4<f32>,
    pub(crate) white_texture: miniquad::Texture,
    pub(crate) canvas: Option<Canvas>,
    pub(crate) sprite_pipeline: miniquad::Pipeline,
    pub(crate) mesh_pipeline: miniquad::Pipeline,

    pub(crate) glyph_brush: GlyphBrush<'static, DrawParam>,
    pub(crate) glyph_cache: Image,
    pub(crate) glyph_state: Rc<RefCell<spritebatch::SpriteBatch>>,
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
            batch_shader::META,
        );

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
                color_blend: Some((
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
            mesh_shader::META,
        );
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
                color_blend: Some((
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
        );

        // Glyph cache stuff.
        let glyph_brush =
            GlyphBrushBuilder::using_font_bytes(Font::default_font_bytes().to_vec()).build();
        let (glyph_cache_width, glyph_cache_height) = glyph_brush.texture_dimensions();
        let initial_contents =
            vec![255; 4 * glyph_cache_width as usize * glyph_cache_height as usize];
        let glyph_cache = Texture::from_rgba8(
            ctx,
            glyph_cache_width as u16,
            glyph_cache_height as u16,
            &initial_contents,
        );

        let glyph_cache = Image::from_texture(ctx, glyph_cache).unwrap();

        let glyph_state = Rc::new(RefCell::new(spritebatch::SpriteBatch::new(
            glyph_cache.clone(),
        )));

        GraphicsContext {
            projection,
            screen_rect,
            white_texture,
            canvas: None,
            sprite_pipeline,
            mesh_pipeline,
            glyph_brush,
            glyph_cache,
            glyph_state,
        }
    }
}

impl GraphicsContext {
    pub fn set_transform(&mut self, _transform: &Matrix3<f32>) {
        unimplemented!();
    }

    pub fn push_transform(&mut self, _transform: &Matrix3<f32>) {
        unimplemented!();
    }

    pub fn pop_transform(&mut self) {
        unimplemented!();
    }

    pub fn set_screen_coordinates(&mut self, rect: crate::graphics::types::Rect) {
        self.screen_rect = rect;
        self.projection =
            cgmath::ortho(rect.x, rect.x + rect.w, rect.y + rect.h, rect.y, -1.0, 1.0);
    }
}

pub(crate) mod batch_shader {
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformType};

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

    pub const META: ShaderMeta = ShaderMeta {
        images: &["Texture"],
        uniforms: UniformBlockLayout {
            uniforms: &[
                ("Projection", UniformType::Mat4),
                ("Model", UniformType::Mat4),
            ],
        },
    };

    #[repr(C)]
    #[derive(Debug)]
    pub struct Uniforms {
        pub projection: cgmath::Matrix4<f32>,
        pub model: cgmath::Matrix4<f32>,
    }
}

pub(crate) mod mesh_shader {
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformType};

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

    pub const META: ShaderMeta = ShaderMeta {
        images: &["Texture"],
        uniforms: UniformBlockLayout {
            uniforms: &[
                ("Projection", UniformType::Mat4),
                ("Model", UniformType::Mat4),
                ("Color", UniformType::Float4),
            ],
        },
    };

    #[repr(C)]
    #[derive(Debug)]
    pub struct Uniforms {
        pub projection: cgmath::Matrix4<f32>,
        pub model: cgmath::Matrix4<f32>,
        pub color: cgmath::Vector4<f32>,
    }
}
