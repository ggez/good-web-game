use crate::{
    conf::NumSamples,
    graphics::{BlendMode, DrawParam, Drawable, FilterMode, Image, Rect},
    Context, GameResult,
};

use miniquad::{RenderPass, Texture, TextureFormat, TextureParams};

#[derive(Clone, Debug)]
pub struct Canvas {
    image: Image,
    pub(crate) offscreen_pass: RenderPass,
}

impl Canvas {
    pub fn new(
        ctx: &mut Context,
        width: u16,
        height: u16,
        _samples: NumSamples,
    ) -> GameResult<Canvas> {
        let texture = Texture::new_render_texture(
            &mut ctx.quad_ctx,
            TextureParams {
                width: width as u32,
                height: height as u32,
                format: TextureFormat::RGBA8,
                ..Default::default()
            },
        );

        // let framebuffer = Framebuffer::new(ctx, &texture)
        //     .ok_or_else(|| GameError::UnknownError("Couldn't create a Framebuffer"))?;
        let image = Image::from_texture(&mut ctx.quad_ctx, texture)?;

        let offscreen_pass = RenderPass::new(&mut ctx.quad_ctx, texture, None);

        Ok(Canvas {
            image,
            offscreen_pass,
        })
    }

    /// Create a new `Canvas` with the current window dimensions.
    pub fn with_window_size(ctx: &mut Context) -> GameResult<Canvas> {
        use crate::graphics;
        let (w, h) = graphics::drawable_size(ctx);
        // Default to no multisampling
        Canvas::new(ctx, w as u16, h as u16, NumSamples::One)
    }

    /// Gets the backend `Image` that is being rendered to.
    pub fn image(&self) -> &Image {
        &self.image
    }

    /// Return the width of the canvas.
    pub fn width(&self) -> u16 {
        self.image.width
    }

    /// Return the height of the canvas.
    pub fn height(&self) -> u16 {
        self.image.height
    }

    /// Returns the dimensions of the canvas.
    pub fn dimensions(&self) -> Rect {
        Rect::new(0.0, 0.0, f32::from(self.width()), f32::from(self.height()))
    }

    /// Get the filter mode for the image.
    pub fn filter(&self) -> FilterMode {
        self.image.filter()
    }

    /// Set the filter mode for the canvas.
    pub fn set_filter(&mut self, mode: FilterMode) {
        self.image.set_filter(mode)
    }

    /// Destroys the `Canvas` and returns the `Image` it contains.
    pub fn into_inner(self) -> Image {
        // TODO: This texture is created with different settings
        // than the default; does that matter?
        self.image
    }
}

impl Drawable for Canvas {
    fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult {
        // No need to flip here. The need for flipping here was just a workaround for canvases
        // being flipped for some reason in ggez. But here they luckily aren't.
        self.image.draw(ctx, param)
    }

    fn set_blend_mode(&mut self, blend_mode: Option<BlendMode>) {
        self.image.set_blend_mode(blend_mode);
    }

    fn blend_mode(&self) -> Option<BlendMode> {
        self.image.blend_mode()
    }

    fn dimensions(&self, _: &mut Context) -> Option<Rect> {
        Some(self.image.dimensions())
    }
}

/// Set the `Canvas` to render to. Specifying `Option::None` will cause all
/// rendering to be done directly to the screen.
pub fn set_canvas(ctx: &mut Context, target: Option<&Canvas>) {
    ctx.gfx_context.canvas = target.cloned();
}
