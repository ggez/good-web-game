use crate::graphics::{types::Rect, Canvas};
//use miniquad_text_rusttype::{FontAtlas, FontTexture};
use std::rc::Rc;

use crate::graphics::{spritebatch, BlendMode, DrawParam, Font, Image};
use cgmath::Matrix4;
use glyph_brush::{GlyphBrush, GlyphBrushBuilder};
use std::cell::RefCell;

pub struct GraphicsContext {
    pub(crate) screen_rect: Rect,
    pub(crate) projection: Matrix4<f32>,
    pub(crate) white_texture: miniquad::Texture,
    pub(crate) canvas: Option<Canvas>,
    pub(crate) sprite_pipeline: miniquad::Pipeline,
    pub(crate) mesh_pipeline: miniquad::Pipeline,
    pub(crate) image_pipeline: miniquad::Pipeline,
    pub(crate) meshbatch_pipeline: miniquad::Pipeline,
    pub(crate) blend_mode: BlendMode,

    pub(crate) glyph_brush: Rc<RefCell<GlyphBrush<DrawParam>>>,
    pub(crate) glyph_cache: Image,
    pub(crate) glyph_state: Rc<RefCell<spritebatch::SpriteBatch>>,
}

impl GraphicsContext {
    pub fn new(ctx: &mut miniquad::Context) -> GraphicsContext {
        use miniquad::*;

        let projection = cgmath::One::one();
        let screen_rect = Rect::new(-1., -1., 2., 2.);

        let white_texture = Texture::from_rgba8(ctx, 1, 1, &[255, 255, 255, 255]);
        let (color_blend, alpha_blend) = BlendMode::Alpha.into();

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
                color_blend: Some(color_blend),
                alpha_blend: Some(alpha_blend),
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
                color_blend: Some(color_blend),
                alpha_blend: Some(alpha_blend),
                ..Default::default()
            },
        );

        let meshbatch_shader = Shader::new(
            ctx,
            meshbatch_shader::VERTEX,
            meshbatch_shader::FRAGMENT,
            meshbatch_shader::meta(),
        )
            .expect("couldn't create mesh batch shader");

        let meshbatch_pipeline = miniquad::Pipeline::with_params(
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
                VertexAttribute::with_buffer("texcoord", VertexFormat::Float2, 0),
                VertexAttribute::with_buffer("color0", VertexFormat::Float4, 0),
                VertexAttribute::with_buffer("Source", VertexFormat::Float4, 1),
                VertexAttribute::with_buffer("Color", VertexFormat::Float4, 1),
                VertexAttribute::with_buffer("InstanceModel", VertexFormat::Mat4, 1),
            ],
            meshbatch_shader,
            PipelineParams {
                color_blend: Some(color_blend),
                alpha_blend: Some(alpha_blend),
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
                color_blend: Some(color_blend),
                alpha_blend: Some(alpha_blend),
                ..Default::default()
            },
        );

        // Glyph cache stuff.
        let font_vec = glyph_brush::ab_glyph::FontArc::try_from_slice(Font::default_font_bytes())
            .expect("Invalid default font bytes, should never happen");
        let glyph_brush = GlyphBrushBuilder::using_font(font_vec).build();
        let (glyph_cache_width, glyph_cache_height) = glyph_brush.texture_dimensions();
        use std::convert::{TryFrom, TryInto};
        let initial_contents = vec![
            255;
            4 * usize::try_from(glyph_cache_width).unwrap()
                * usize::try_from(glyph_cache_height).unwrap()
        ];
        let glyph_cache = Texture::from_rgba8(
            ctx,
            glyph_cache_width.try_into().unwrap(),
            glyph_cache_height.try_into().unwrap(),
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
            image_pipeline,
            meshbatch_pipeline,
            blend_mode: BlendMode::Alpha,
            glyph_brush: Rc::new(RefCell::new(glyph_brush)),
            glyph_cache,
            glyph_state,
        }
    }
}

impl GraphicsContext {
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

    pub(crate) fn set_blend_mode(&mut self, mode: BlendMode) {
        self.blend_mode = mode;
    }

    /// Gets the current global
    pub(crate) fn blend_mode(&self) -> &BlendMode {
        &self.blend_mode
    }
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

// TODO: this shader is WIP (probably)
pub(crate) mod meshbatch_shader {
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformDesc, UniformType};

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 position;
    attribute vec2 texcoord;
    attribute vec4 color0;
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
        color = Color * color0;
        uv = texcoord * Source.zw + Source.xy;
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

pub(crate) mod mesh_shader {
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformDesc, UniformType};

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 position;
    attribute vec2 texcoord;
    attribute vec4 color0;

    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform mat4 Projection;
    uniform vec4 Source;
    uniform mat4 Model;
    uniform vec4 Color;

    uniform float depth;

    void main() {
        gl_Position = Projection * Model * vec4(position, 0, 1);
        gl_Position.z = depth;
        color = Color * color0;
        uv = texcoord * Source.zw + Source.xy;
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
        pub source: cgmath::Vector4<f32>,
        pub model: cgmath::Matrix4<f32>,
        pub color: cgmath::Vector4<f32>,
    }
}
