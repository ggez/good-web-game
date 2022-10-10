use crate::graphics::{types::Rect, Canvas, FilterMode, Shader, ShaderId};
use std::rc::Rc;

use crate::graphics::{spritebatch, BlendMode, DrawParam, Font, Image};
use cgmath::Matrix4;
use glyph_brush::{GlyphBrush, GlyphBrushBuilder};
use miniquad::{BufferLayout, PipelineParams, Texture, VertexAttribute, VertexFormat, VertexStep};
use std::cell::RefCell;

pub struct GraphicsContext {
    pub(crate) screen_rect: Rect,
    pub(crate) projection: Matrix4<f32>,
    pub(crate) white_texture: miniquad::Texture,
    pub(crate) canvas: Option<Canvas>,
    pub(crate) current_shader: Rc<RefCell<ShaderId>>,
    pub(crate) shaders: Vec<Shader>,
    pub(crate) blend_mode: BlendMode,
    pub(crate) default_filter: FilterMode,

    pub(crate) glyph_brush: Rc<RefCell<GlyphBrush<DrawParam>>>,
    pub(crate) glyph_cache: Image,
    pub(crate) glyph_state: Rc<RefCell<spritebatch::SpriteBatch>>,
}

impl GraphicsContext {
    pub fn new(quad_ctx: &mut miniquad::graphics::GraphicsContext) -> GraphicsContext {
        let projection = cgmath::One::one();
        let screen_rect = Rect::new(-1., -1., 2., 2.);

        let white_texture = Texture::from_rgba8(quad_ctx, 1, 1, &[255, 255, 255, 255]);
        let (color_blend, alpha_blend) = BlendMode::Alpha.into();

        let default_shader = miniquad::Shader::new(
            quad_ctx,
            default_shader::VERTEX,
            default_shader::FRAGMENT,
            default_shader::meta(),
        )
        .expect("couldn't create default shader");

        let gwg_default_shader =
            crate::graphics::Shader::from_mini_shader(quad_ctx, default_shader, None);

        let default_pipeline = miniquad::Pipeline::with_params(
            quad_ctx,
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
                VertexAttribute::with_buffer("Model", VertexFormat::Mat4, 1),
            ],
            default_shader,
            PipelineParams {
                color_blend: Some(color_blend),
                alpha_blend: Some(alpha_blend),
                ..Default::default()
            },
        );

        // pipeline has to be applied whenever the shader (and therefore pipeline) changes
        quad_ctx.apply_pipeline(&default_pipeline);

        // Glyph cache stuff.
        let font_vec = glyph_brush::ab_glyph::FontArc::try_from_slice(Font::default_font_bytes())
            .expect("Invalid default font bytes, should never happen");
        let glyph_brush = GlyphBrushBuilder::using_font(font_vec).build();
        let (glyph_cache_width, glyph_cache_height) = glyph_brush.texture_dimensions();
        let initial_contents = vec![
            255;
            4 * usize::try_from(glyph_cache_width).unwrap()
                * usize::try_from(glyph_cache_height).unwrap()
        ];
        let glyph_cache = Texture::from_rgba8(
            quad_ctx,
            glyph_cache_width.try_into().unwrap(),
            glyph_cache_height.try_into().unwrap(),
            &initial_contents,
        );

        let glyph_cache = Image::from_texture(quad_ctx, glyph_cache, FilterMode::Linear).unwrap();

        let glyph_state = Rc::new(RefCell::new(spritebatch::SpriteBatch::new(
            glyph_cache.clone(),
        )));

        GraphicsContext {
            projection,
            screen_rect,
            white_texture,
            canvas: None,
            current_shader: Rc::new(RefCell::new(0)),
            shaders: vec![gwg_default_shader],
            blend_mode: BlendMode::Alpha,
            default_filter: FilterMode::Linear,
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

pub(crate) mod default_shader {
    use crate::graphics::ShaderId;
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformDesc, UniformType};

    /// As the default shader is created first it holds the first id, which is 0.
    pub const SHADER_ID: ShaderId = 0;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 position;
    attribute vec2 texcoord;
    attribute vec4 color0;

    attribute vec4 Source;
    attribute vec4 Color;
    attribute mat4 Model;

    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform mat4 Projection;

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
                uniforms: vec![UniformDesc::new("Projection", UniformType::Mat4)],
            },
        }
    }
}
