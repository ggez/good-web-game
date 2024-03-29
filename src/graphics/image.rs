use cgmath::Matrix4;
use std::path;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
    error::GameResult,
    filesystem,
    graphics::{BlendMode, DrawParam, Drawable, InstanceAttributes, Rect},
    Context, GameError,
};

use miniquad::{Bindings, Buffer, BufferType, PassAction, Texture};

use crate::graphics::{apply_uniforms, Color};
pub use miniquad::graphics::FilterMode;
use std::sync::Arc;

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
    pub fn new<P: AsRef<path::Path>>(
        ctx: &mut Context,
        quad_ctx: &mut miniquad::graphics::GraphicsContext,
        path: P,
    ) -> GameResult<Self> {
        use std::io::Read;

        let mut file = filesystem::open(ctx, path)?;

        let mut bytes = vec![];
        file.bytes.read_to_end(&mut bytes)?;

        Self::from_png_bytes(ctx, quad_ctx, &bytes)
    }

    pub fn from_png_bytes(
        ctx: &mut Context,
        quad_ctx: &mut miniquad::graphics::GraphicsContext,
        bytes: &[u8],
    ) -> GameResult<Self> {
        match image::load_from_memory(bytes) {
            Ok(img) => {
                let rgba = img.to_rgba();

                let width = rgba.width() as u16;
                let height = rgba.height() as u16;
                let bytes = rgba.into_raw();

                Image::from_rgba8(ctx, quad_ctx, width, height, &bytes)
            }
            Err(e) => Err(GameError::ResourceLoadError(e.to_string())),
        }
    }

    pub fn from_rgba8(
        ctx: &mut Context,
        quad_ctx: &mut miniquad::graphics::GraphicsContext,
        width: u16,
        height: u16,
        bytes: &[u8],
    ) -> GameResult<Image> {
        let texture = Texture::from_rgba8(quad_ctx, width, height, bytes);
        Self::from_texture(quad_ctx, texture, ctx.gfx_context.default_filter)
    }

    pub fn from_texture(
        ctx: &mut miniquad::Context,
        texture: Texture,
        filter: FilterMode,
    ) -> GameResult<Image> {
        // if we wanted to we could optimize this a bit by creating this buffer only once and then just cloning the handle
        #[rustfmt::skip]
        let vertices: [f32; 32] = [0.0, 0.0, // first pos
                                  0.0, 0.0, // first texcoord
                                  1.0, 1.0, 1.0, 1.0, // first color
                                  1.0, 0.0, // second pos
                                  1.0, 0.0, // second texcoord
                                  1.0, 1.0, 1.0, 1.0, // second color
                                  1.0, 1.0, // third pos
                                  1.0, 1.0, // third texcoord
                                  1.0, 1.0, 1.0, 1.0, // third color
                                  0.0, 1.0, // fourth pos
                                  0.0, 1.0, // fourth texcoord
                                  1.0, 1.0, 1.0, 1.0]; // fourth color

        let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);
        let attribute_buffer = Buffer::stream(
            ctx,
            BufferType::VertexBuffer,
            std::mem::size_of::<InstanceAttributes>(), // start out with space for one instance
        );

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer, attribute_buffer],
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
    pub fn solid(
        context: &mut Context,
        quad_ctx: &mut miniquad::graphics::GraphicsContext,
        size: u16,
        color: Color,
    ) -> GameResult<Self> {
        let (r, g, b, a) = color.into();
        let pixel_array: [u8; 4] = [r, g, b, a];
        let size_squared = usize::from(size) * usize::from(size);
        let mut buffer = Vec::with_capacity(size_squared);
        for _i in 0..size_squared {
            buffer.extend(&pixel_array[..]);
        }
        Image::from_rgba8(context, quad_ctx, size, size, &buffer)
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
    pub(crate) fn draw_image_raw(
        &self,
        ctx: &mut Context,
        quad_ctx: &mut miniquad::graphics::GraphicsContext,
        param: DrawParam,
    ) -> GameResult {
        let instance = InstanceAttributes::from(&param);
        self.bindings.vertex_buffers[1].update(quad_ctx, &[instance]);

        if self.dirty_filter.load() {
            self.dirty_filter.store(false);
            self.texture.set_filter(quad_ctx, self.filter);
        }

        let pass = ctx.framebuffer();
        quad_ctx.begin_pass(pass, PassAction::Nothing);
        quad_ctx.apply_bindings(&self.bindings);

        let shader_id = *ctx.gfx_context.current_shader.borrow();
        let current_shader = &mut ctx.gfx_context.shaders[shader_id];
        quad_ctx.apply_pipeline(&current_shader.pipeline);

        apply_uniforms(ctx, quad_ctx, shader_id, None);

        let mut custom_blend = false;
        if let Some(blend_mode) = self.blend_mode() {
            custom_blend = true;
            crate::graphics::set_current_blend_mode(quad_ctx, blend_mode)
        }

        quad_ctx.draw(0, 6, 1);

        // restore default blend mode
        if custom_blend {
            crate::graphics::restore_blend_mode(ctx, quad_ctx);
        }

        quad_ctx.end_render_pass();

        Ok(())
    }
}

impl Drawable for Image {
    fn draw(
        &self,
        ctx: &mut Context,
        quad_ctx: &mut miniquad::graphics::GraphicsContext,
        param: DrawParam,
    ) -> GameResult {
        let src_width = param.src.w;
        let src_height = param.src.h;
        // We have to mess with the scale to make everything
        // be its-unit-size-in-pixels.
        let scale_x = src_width * f32::from(self.width);
        let scale_y = src_height * f32::from(self.height);

        let new_param = match param.trans {
            crate::graphics::Transform::Values { scale, .. } => param.scale(mint::Vector2 {
                x: scale.x * scale_x,
                y: scale.y * scale_y,
            }),
            crate::graphics::Transform::Matrix(m) => param.transform(
                Matrix4::from(m) * Matrix4::from_nonuniform_scale(scale_x, scale_y, 1.0),
            ),
        };

        self.draw_image_raw(ctx, quad_ctx, new_param)
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
