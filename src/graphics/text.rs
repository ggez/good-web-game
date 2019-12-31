use super::{BlendMode, Color, DrawParam, Drawable, GameResult, Rect};

use crate::graphics::context::GpuText;

use cgmath::{Matrix4, Point2, Vector2, Vector3, Vector4};

use miniquad::*;

use std::cell::RefCell;

thread_local! {
    static FONTS: RefCell<Vec<String>> = RefCell::new(vec![]);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontId {
    TtfFontId,
    CanvasFontId(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Font(pub FontId);

impl Font {
    /// Should construct font from the ttf file path.
    pub fn new(_: &mut crate::Context, _: &str) -> GameResult<Font> {
        Ok(Font(FontId::TtfFontId))
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct Scale {
    /// Horizontal scale, in pixels.
    pub x: f32,
    /// Vertical scale, in pixels.
    pub y: f32,
}

impl Scale {
    /// Uniform scaling, equivalent to `Scale { x: s, y: s }`.
    #[inline]
    pub fn uniform(s: f32) -> Scale {
        Scale { x: s, y: s }
    }
}

/// A piece of text with optional color, font and font scale information.
/// Drawing text generally involves one or more of these.
/// These options take precedence over any similar field/argument.
/// Can be implicitly constructed from `String`, `(String, Color)`, and `(String, FontId, Scale)`.
#[derive(Clone, Debug)]
pub struct TextFragment {
    /// Text string itself.
    pub text: String,
    /// Fragment's color, defaults to text's color.
    pub color: Option<Color>,
    /// Fragment's font, defaults to text's font.
    pub font: Option<Font>,
    /// Fragment's scale, defaults to text's scale.
    pub scale: Option<Scale>,
}

impl Default for TextFragment {
    fn default() -> Self {
        TextFragment {
            text: "".into(),
            color: None,
            font: None,
            scale: None,
        }
    }
}

impl TextFragment {
    /// Creates a new fragment from `String` or `&str`.
    pub fn new<T: Into<Self>>(text: T) -> Self {
        text.into()
    }

    /// Set fragment's color, overrides text's color.
    pub fn color(mut self, color: Color) -> TextFragment {
        self.color = Some(color);
        self
    }

    /// Set fragment's font, overrides text's font.
    pub fn font(mut self, font: Font) -> TextFragment {
        self.font = Some(font);
        self
    }

    /// Set fragment's scale, overrides text's scale.
    pub fn scale(mut self, scale: Scale) -> TextFragment {
        self.scale = Some(scale);
        self
    }
}

impl<'a> From<&'a str> for TextFragment {
    fn from(text: &'a str) -> TextFragment {
        TextFragment {
            text: text.to_owned(),
            ..Default::default()
        }
    }
}

impl From<char> for TextFragment {
    fn from(ch: char) -> TextFragment {
        TextFragment {
            text: ch.to_string(),
            ..Default::default()
        }
    }
}

impl From<String> for TextFragment {
    fn from(text: String) -> TextFragment {
        TextFragment {
            text,
            ..Default::default()
        }
    }
}

impl<T> From<(T, Font, f32)> for TextFragment
where
    T: Into<TextFragment>,
{
    fn from((text, font, scale): (T, Font, f32)) -> TextFragment {
        text.into().font(font).scale(Scale::uniform(scale))
    }
}

#[derive(Debug, Clone)]
pub struct Text {
    fragment: TextFragment,
}

impl Text {
    /// Creates a `Text` from a `TextFragment`.
    ///
    /// ```rust
    /// # use ggez::graphics::Text;
    /// # fn main() {
    /// let text = Text::new("foo");
    /// # }
    /// ```
    pub fn new<F>(fragment: F) -> Text
    where
        F: Into<TextFragment>,
    {
        Text {
            fragment: fragment.into(),
        }
    }

    fn measure_dimensions(&self, ctx: &mut crate::Context) -> Rect {
        // let dimensions = ctx
        //     .gfx_context
        //     .canvas_context
        //     .measure_label(&self.fragment.text, None);
        //Rect::new(0., 0., dimensions.x as f32, dimensions.y as f32)
        Rect::new(0., 0., self.fragment.text.len() as f32 * 10.0, 10.0)
    }
}

fn load_gpu_text(ctx: &mut crate::Context, label: &str) -> GpuText {
    let shader = Shader::new(
        &mut ctx.quad_ctx,
        text_shader::VERTEX,
        text_shader::FRAGMENT,
        text_shader::META,
    );

    let mut vertices = Vec::<f32>::new();
    let mut indices = Vec::<u16>::new();
    for (n, ch) in label.chars().enumerate() {
        let ix = ch as u32;

        let sx = ((ix % 16) as f32) / 16.0;
        let sy = ((ix / 16) as f32) / 16.0;
        let sw = 1.0 / 16.0;
        let sh = 1.0 / 16.0;

        #[rustfmt::skip]
        let letter: [f32; 16] = [
            0.0 + n as f32 * 10., 0.0, sx, sy, 
            10.0 + n as f32 * 10., 0.0, sx + sw, sy, 
            10.0 + n as f32 * 10., 10.0, sx + sw, sy + sh, 
            0.0 + n as f32 * 10., 10.0, sx, sy + sh
        ];
        vertices.extend(letter.iter());
        let n = n as u16;
        indices.extend(
            [
                n * 4 + 0,
                n * 4 + 1,
                n * 4 + 2,
                n * 4 + 0,
                n * 4 + 2,
                n * 4 + 3,
            ]
            .iter()
            .map(|x| *x),
        );
    }

    let vertex_buffer = unsafe {
        Buffer::immutable(
            &mut ctx.quad_ctx,
            BufferType::VertexBuffer,
            &vertices,
        )
    };

    let index_buffer = unsafe {
        Buffer::immutable(
            &mut ctx.quad_ctx,
            BufferType::IndexBuffer,
            &indices,
        )
    };

    let bindings = Bindings {
        vertex_buffers: vec![vertex_buffer],
        index_buffer: index_buffer,
        images: vec![ctx.internal.gfx_context.font_texture.clone()],
    };

    let pipeline = Pipeline::with_params(
        &mut ctx.quad_ctx,
        &[BufferLayout::default()],
        &[
            VertexAttribute::new("position", VertexFormat::Float2),
            VertexAttribute::new("texcoord", VertexFormat::Float2),
        ],
        shader,
        PipelineParams {
            color_blend: Some((
                Equation::Add,
                BlendFactor::Value(BlendValue::SourceAlpha),
                BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
            )),
            ..Default::default()
        },
    );

    GpuText { bindings, pipeline }
}

impl Drawable for Text {
    fn draw(&self, ctx: &mut crate::Context, param: DrawParam) -> GameResult {
        let real_size = Vector2::new(param.src.w as f32, param.src.h as f32);
        let size = Matrix4::from_nonuniform_scale(
            real_size.x * param.scale.x,
            real_size.y * param.scale.y,
            0.,
        );
        let dest = Point2::new(
            param.dest.x - real_size.x * param.offset.x * param.scale.x,
            param.dest.y - real_size.y * param.offset.y * param.scale.y,
        );
        let pos = Matrix4::from_translation(Vector3::new(dest.x, dest.y, 0.));
        let transform = pos * size;

        if ctx.text_cache().contains_key(&self.fragment.text) == false {
            let text = load_gpu_text(ctx, &self.fragment.text);
            ctx.text_cache().insert(self.fragment.text.clone(), text);
        }
        let pass = ctx.framebuffer();
        ctx.quad_ctx.begin_pass(pass, PassAction::Nothing);

        let text = ctx
            .internal
            .gfx_context
            .text_cache
            .get(&self.fragment.text.clone())
            .unwrap();

        ctx.quad_ctx.apply_pipeline(&text.pipeline);
        ctx.quad_ctx.apply_bindings(&text.bindings);

        let uniforms = text_shader::Uniforms {
            projection: ctx.internal.gfx_context.projection,
            model: transform,
            color: Vector4::new(param.color.r, param.color.g, param.color.b, param.color.a),
        };
        unsafe {
            ctx.quad_ctx.apply_uniforms(&uniforms);
        }

        // TODO: buffer len from miniquad?
        ctx.quad_ctx.draw(0, self.fragment.text.len() as i32 * 6, 1);

        ctx.quad_ctx.end_render_pass();

        Ok(())
    }

    fn dimensions(&self, ctx: &mut crate::Context) -> Option<Rect> {
        Some(self.measure_dimensions(ctx))
    }

    fn set_blend_mode(&mut self, _: Option<BlendMode>) {
        unimplemented!()
    }

    fn blend_mode(&self) -> Option<BlendMode> {
        unimplemented!()
    }
}

mod text_shader {
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformType};

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 position;
    attribute vec2 texcoord;

    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform mat4 Projection;
    uniform mat4 Model;
    uniform vec4 Color;

    uniform float depth;

    void main() {
        gl_Position = Projection * Model * vec4(position, 0, 1);
        gl_Position.z = depth;
        color = Color;
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
