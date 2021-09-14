use cgmath::{Matrix4, Transform, Vector4};
use std::path;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
    error::GameResult,
    filesystem,
    graphics::{context::image_shader, BlendMode, DrawParam, Drawable, Rect},
    Context, GameError,
};

use miniquad::{Bindings, Buffer, BufferType, PassAction, Texture};

use crate::graphics::Color;
pub use miniquad::graphics::FilterMode;
use std::sync::Arc;

#[derive(Debug, Clone)]
#[repr(C)]
pub(crate) struct InstanceAttributes {
    pub source: Vector4<f32>,
    pub color: Vector4<f32>,
    pub model: Matrix4<f32>,
}

impl Default for InstanceAttributes {
    fn default() -> InstanceAttributes {
        InstanceAttributes {
            source: Vector4::new(0., 0., 0., 0.),
            color: Vector4::new(0., 0., 0., 0.),
            model: Matrix4::one(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Image {
    pub(crate) texture: Texture,
    pub(crate) width: u16,
    pub(crate) height: u16,
    filter: FilterMode,
    pub(crate) bindings: Bindings,
    blend_mode: Option<BlendMode>,
    dirty_filter: DirtyFlag,
    pub(crate) bindings_clones_hack: Arc<()>,
    pub(crate) texture_clones_hack: Arc<()>,
}

impl Image {
    pub fn new<P: AsRef<path::Path>>(ctx: &mut Context, path: P) -> GameResult<Self> {
        use std::io::Read;

        let mut file = filesystem::open(ctx, path)?;

        let mut bytes = vec![];
        file.bytes.read_to_end(&mut bytes)?;

        Self::from_png_bytes(ctx, &bytes)
    }

    pub fn from_png_bytes(ctx: &mut Context, bytes: &[u8]) -> GameResult<Self> {
        match image::load_from_memory(bytes) {
            Ok(img) => {
                let rgba = img.to_rgba();

                let width = rgba.width() as u16;
                let height = rgba.height() as u16;
                let bytes = rgba.into_raw();

                Image::from_rgba8(ctx, width, height, &bytes)
            }
            Err(e) => Err(GameError::ResourceLoadError(e.to_string())),
        }
    }

    pub fn from_rgba8(
        ctx: &mut Context,
        width: u16,
        height: u16,
        bytes: &[u8],
    ) -> GameResult<Image> {
        let texture = Texture::from_rgba8(&mut ctx.quad_ctx, width, height, bytes);

        Self::from_texture(&mut ctx.quad_ctx, texture, ctx.gfx_context.default_filter)
    }

    pub fn from_texture(
        ctx: &mut miniquad::Context,
        texture: Texture,
        filter: FilterMode,
    ) -> GameResult<Image> {
        #[rustfmt::skip]
        let vertices: [f32; 8] = [0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0];
        let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![texture],
        };

        Ok(Image {
            width: texture.width as u16,
            height: texture.height as u16,
            texture,
            bindings,
            blend_mode: None,
            dirty_filter: DirtyFlag::new(false),
            filter,
            bindings_clones_hack: Arc::new(()),
            texture_clones_hack: Arc::new(()),
        })
    }

    /// A little helper function that creates a new `Image` that is just
    /// a solid square of the given size and color.  Mainly useful for
    /// debugging.
    pub fn solid(context: &mut Context, size: u16, color: Color) -> GameResult<Self> {
        let (r, g, b, a) = color.into();
        let pixel_array: [u8; 4] = [r, g, b, a];
        let size_squared = usize::from(size) * usize::from(size);
        let mut buffer = Vec::with_capacity(size_squared);
        for _i in 0..size_squared {
            buffer.extend(&pixel_array[..]);
        }
        Image::from_rgba8(context, size, size, &buffer)
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
        self.dirty_filter.store(true);
        self.filter = filter;
    }

    pub fn filter(&self) -> FilterMode {
        self.filter
    }

    /// Draws without adapting the scaling.
    pub(crate) fn draw_image_raw(&self, ctx: &mut Context, param: DrawParam) -> GameResult {
        let transform = param.trans.to_bare_matrix().into();

        if self.dirty_filter.load() {
            self.dirty_filter.store(false);
            self.texture.set_filter(&mut ctx.quad_ctx, self.filter);
        }

        let pass = ctx.framebuffer();
        ctx.quad_ctx.begin_pass(pass, PassAction::Nothing);
        ctx.quad_ctx.apply_pipeline(&ctx.gfx_context.image_pipeline);
        ctx.quad_ctx.apply_bindings(&self.bindings);

        let uniforms = image_shader::Uniforms {
            projection: ctx.gfx_context.projection,
            model: transform,
            source: Vector4::new(param.src.x, param.src.y, param.src.w, param.src.h),
            color: Vector4::new(param.color.r, param.color.g, param.color.b, param.color.a),
        };

        ctx.quad_ctx.apply_uniforms(&uniforms);

        let mut custom_blend = false;
        if let Some(blend_mode) = self.blend_mode() {
            custom_blend = true;
            crate::graphics::set_current_blend_mode(ctx, blend_mode)
        }

        ctx.quad_ctx.draw(0, 6, 1);

        // restore default blend mode
        if custom_blend {
            crate::graphics::restore_blend_mode(ctx);
        }

        ctx.quad_ctx.end_render_pass();

        Ok(())
    }
}

impl Drawable for Image {
    fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult {
        let src_width = param.src.w;
        let src_height = param.src.h;
        // We have to mess with the scale to make everything
        // be its-unit-size-in-pixels.
        let scale_x = src_width * f32::from(self.width);
        let scale_y = src_height * f32::from(self.height);

        use crate::graphics::Transform;
        let new_param = match param.trans {
            Transform::Values { scale, .. } => param.scale(mint::Vector2 {
                x: scale.x * scale_x,
                y: scale.y * scale_y,
            }),
            Transform::Matrix(m) => param.transform(
                Matrix4::from(m) * Matrix4::from_nonuniform_scale(scale_x, scale_y, 1.0),
            ),
        };

        self.draw_image_raw(ctx, new_param)
    }

    fn set_blend_mode(&mut self, mode: Option<BlendMode>) {
        self.blend_mode = mode;
    }

    /// Gets the blend mode to be used when drawing this drawable.
    fn blend_mode(&self) -> Option<BlendMode> {
        self.blend_mode
    }

    fn dimensions(&self, _ctx: &mut Context) -> Option<Rect> {
        Some(self.dimensions())
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        if Arc::strong_count(&self.bindings_clones_hack) == 1 {
            crate::graphics::add_dropped_bindings(
                self.bindings.clone(),
                Arc::strong_count(&self.texture_clones_hack) == 1,
            );
        }
    }
}

#[derive(Debug)]
struct DirtyFlag(AtomicBool);

impl DirtyFlag {
    pub fn new(value: bool) -> Self {
        Self(AtomicBool::new(value))
    }

    pub fn load(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    pub fn store(&self, value: bool) {
        self.0.store(value, Ordering::Release)
    }
}

impl Clone for DirtyFlag {
    fn clone(&self) -> Self {
        DirtyFlag(AtomicBool::new(self.0.load(Ordering::Acquire)))
    }
}
