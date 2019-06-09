use crate::{
    Context,
    GameResult,
    conf::NumSamples,
    error::GameError,
    graphics::{Drawable, DrawParam, Rect, BlendMode, FilterMode, Image, context::webgl::{Texture, Framebuffer}},
};

pub struct Canvas {
    image: Image,
    framebuffer: Framebuffer,
}

impl Canvas {
    pub fn new(
        ctx: &mut Context,
        width: u16,
        height: u16,
        samples: NumSamples
    ) -> GameResult<Canvas> {
        let texture = Texture::from_rgba8(ctx, width, height, None);
        let framebuffer = Framebuffer::new(ctx, &texture)
                            .ok_or_else(|| GameError::UnknownError("Couldn't create a Framebuffer"))?;
        let image = Image::from_texture(width, height, Some(texture));

        Ok(Canvas {
            image,
            framebuffer,
        })
    }

    /// Create a new `Canvas` with the current window dimensions.
    pub fn with_window_size(ctx: &mut Context) -> GameResult<Canvas> {
        use crate::graphics;
        let (w, h) = graphics::drawable_size(ctx);
        // Default to no multisampling
        // TODO: Use winit's into() to translate f64's more accurately
        // ...where the heck IS winit's into()?  wth was I referring to?
        Canvas::new(ctx, w as u16, h as u16, NumSamples::One)
    }

    /// Gets the backend `Image` that is being rendered to.
    pub fn image(&self) -> &Image {
        &self.image
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
        // Gotta flip the image on the Y axis here
        // to account for OpenGL's origin being at the bottom-left.
        let mut flipped_param = param;
        flipped_param.scale.y *= -1.0;
        flipped_param.dest.y += self.image.height() as f32 * param.scale.y;
        self.image.draw(ctx, flipped_param)
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
    let (width, height) = target.map(|canvas| {
        // Use dimensions of the image bound to framebuffer
        let rect = canvas.image().dimensions();

        (rect.w as u32, rect.h as u32)
    }).unwrap_or_else(|| {
        // Use original dimensions of the webgl canvas context
        let (width, height) = ctx.gfx_context.canvas_context.size();

        (width as u32, height as u32)
    });

    let webgl = &mut ctx.gfx_context.webgl_context;

    webgl.set_framebuffer(target.map(|canvas| &canvas.framebuffer));
    webgl.resize(width, height);
}
