use cgmath::{Matrix4, Point2, Vector2, Vector3, Vector4};
use std::{cell::Cell, path};

use crate::{
    error::{GameError, GameResult},
    filesystem,
    filesystem::File,
    graphics::{BlendMode, DrawParam, Drawable, Rect},
    Context,
};

use miniquad::{
    Bindings, BlendFactor, BlendValue, Buffer, BufferType, Equation, PassAction, Pipeline,
    PipelineParams, Shader, Texture, Usage, VertexAttribute, VertexFormat, VertexLayout,
};

#[derive(Clone, Debug)]
pub struct Image {
    texture: Texture,
    width: u16,
    height: u16,
    filter: FilterMode,
    bindings: Bindings,
    pipeline: Pipeline,
    dirty_filter: Cell<bool>,
}

#[derive(Clone, Copy, Debug)]
pub enum FilterMode {
    Linear,  // = 0LINEAR_FILTER as isize,
    Nearest, // = NEAREST_FILTER as isize,
}

impl Image {
    pub fn new<P: AsRef<path::Path>>(ctx: &mut Context, path: P) -> GameResult<Self> {
        use std::io::Read;

        let mut file = filesystem::open(ctx, path)?;

        let mut bytes = vec![];
        file.bytes.read_to_end(&mut bytes);

        Self::from_png_bytes(ctx, &bytes)
    }

    pub fn from_png_bytes(ctx: &mut Context, bytes: &[u8]) -> GameResult<Self> {
        let img = image::load_from_memory(&bytes)
            .unwrap_or_else(|e| panic!(e))
            .to_rgba();
        let width = img.width() as u16;
        let height = img.height() as u16;
        let bytes = img.into_raw();

        Image::from_rgba8(ctx, width, height, &bytes)
    }

    pub fn from_rgba8(
        ctx: &mut Context,
        width: u16,
        height: u16,
        bytes: &[u8],
    ) -> GameResult<Image> {
        let texture = Texture::from_rgba8(width, height, bytes);

        Self::from_texture(ctx, texture)
    }

    pub fn from_texture(ctx: &mut Context, texture: Texture) -> GameResult<Image> {
        #[rustfmt::skip]
        let vertices: [f32; 8] = [0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0];
        let vertex_buffer = unsafe {
            Buffer::new(
                &mut ctx.quad_ctx,
                BufferType::VertexBuffer,
                Usage::Immutable,
                &vertices,
            )
        };

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = unsafe {
            Buffer::new(
                &mut ctx.quad_ctx,
                BufferType::IndexBuffer,
                Usage::Immutable,
                &indices,
            )
        };

        let bindings = Bindings {
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
            images: vec![texture],
        };

        let shader = Shader::new(
            &mut ctx.quad_ctx,
            image_shader::VERTEX,
            image_shader::FRAGMENT,
            image_shader::META,
        );

        let pipeline = Pipeline::with_params(
            &mut ctx.quad_ctx,
            VertexLayout::new(&[(VertexAttribute::Custom("position"), VertexFormat::Float2)]),
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

        Ok(Image {
            width: texture.width as u16,
            height: texture.height as u16,
            texture,
            bindings,
            pipeline,
            dirty_filter: Cell::new(false),
            filter: FilterMode::Linear,
        })
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    /// Returns the dimensions of the image.
    pub fn dimensions(&self) -> Rect {
        Rect::new(0.0, 0.0, self.width() as f32, self.height() as f32)
    }

    pub fn set_filter(&mut self, filter: FilterMode) {
        self.dirty_filter.set(true);
        self.filter = filter;
    }

    pub fn filter(&self) -> FilterMode {
        self.filter
    }
}

impl Drawable for Image {
    fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult {
        let real_size = Vector2::new(
            param.src.w * self.width as f32,
            param.src.h * self.height as f32,
        );
        let size_vec = Vector2::new(real_size.x * param.scale.x, real_size.y * param.scale.y);
        let size = Matrix4::from_nonuniform_scale(size_vec.x, size_vec.y, 0.);
        let dest = Point2::new(
            param.dest.x - real_size.x * param.offset.x * param.scale.x,
            param.dest.y - real_size.y * param.offset.y * param.scale.y,
        );
        let pos = Matrix4::from_translation(Vector3::new(
            dest.x + size_vec.x / 2.,
            dest.y + size_vec.y / 2.,
            0.,
        ));
        let rot = Matrix4::from_angle_z(cgmath::Rad(param.rotation));
        let pos0 = Matrix4::from_translation(Vector3::new(-size_vec.x / 2., -size_vec.y / 2., 0.));
        let transform = pos * rot * pos0 * size;

        if self.dirty_filter.get() {
            self.dirty_filter.set(false);

            self.texture.set_filter(self.filter as i32);
        }

        let pass = ctx.framebuffer();
        ctx.quad_ctx.begin_pass(pass, PassAction::Nothing);
        ctx.quad_ctx.apply_pipeline(&self.pipeline);
        ctx.quad_ctx.apply_bindings(&self.bindings);

        let uniforms = image_shader::Uniforms {
            projection: ctx.internal.gfx_context.projection,
            model: transform,
            source: Vector4::new(param.src.x, param.src.y, param.src.w, param.src.h),
            color: Vector4::new(param.color.r, param.color.g, param.color.b, param.color.a),
        };
        unsafe {
            ctx.quad_ctx.apply_uniforms(&uniforms);
        }
        ctx.quad_ctx.draw(6);

        ctx.quad_ctx.end_render_pass();

        Ok(())
    }

    fn set_blend_mode(&mut self, _: Option<BlendMode>) {}

    /// Gets the blend mode to be used when drawing this drawable.
    fn blend_mode(&self) -> Option<BlendMode> {
        unimplemented!()
    }

    fn dimensions(&self, _ctx: &mut Context) -> Option<Rect> {
        Some(self.dimensions())
    }
}

mod image_shader {
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformType};

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 position;
    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform mat4 Projection;
    uniform mat4 Model;
    uniform vec4 Source;
    uniform vec4 Color;

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

    pub const META: ShaderMeta = ShaderMeta {
        images: &["Texture"],
        uniforms: UniformBlockLayout {
            uniforms: &[
                ("Projection", UniformType::Mat4),
                ("Model", UniformType::Mat4),
                ("Source", UniformType::Float4),
                ("Color", UniformType::Float4),
            ],
        },
    };

    #[repr(C)]
    #[derive(Debug)]
    pub struct Uniforms {
        pub projection: cgmath::Matrix4<f32>,
        pub model: cgmath::Matrix4<f32>,
        pub source: cgmath::Vector4<f32>,
        pub color: cgmath::Vector4<f32>,
    }
}
