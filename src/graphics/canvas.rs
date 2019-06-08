use crate::{
    Context,
    GameResult,
    conf::NumSamples,
    error::GameError,
    graphics::{Drawable, DrawParam, Rect, BlendMode, Image, context::webgl::{Texture, Framebuffer}},
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

    fn dimensions(&self) -> Rect {
        self.image.dimensions()
    }
}

impl Drawable for Canvas {
    fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult {
        self.image.draw(ctx, param)
    }

    fn set_blend_mode(&mut self, blend_mode: Option<BlendMode>) {
        self.image.set_blend_mode(blend_mode);
    }

    fn blend_mode(&self) -> Option<BlendMode> {
        self.image.blend_mode()
    }

    fn dimensions(&self, _: &mut Context) -> Option<Rect> {
        Some(self.dimensions())
    }
}

/// Set the `Canvas` to render to. Specifying `Option::None` will cause all
/// rendering to be done directly to the screen.
pub fn set_canvas(ctx: &mut Context, target: Option<&Canvas>) {
    let (width, height) = target.map(|canvas| {
        // Use dimensions of the image bound to framebuffer
        let rect = canvas.dimensions();

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
