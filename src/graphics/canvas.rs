use crate::{
    graphics::{BlendMode, DrawParam, Drawable, FilterMode, Image, Rect},
    Context, GameResult,
};

use crate::graphics::drawparam::Transform;
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
        Canvas::new(ctx, w as u16, h as u16)
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
        // We have to mess with the scale to make everything
        // be its-unit-size-in-pixels.
        let scale_x = param.src.w * f32::from(self.width());
        let scale_y = param.src.h * f32::from(self.height());

        let param = param.transform(
            cgmath::Matrix4::from(param.trans.to_bare_matrix())
                * cgmath::Matrix4::from_nonuniform_scale(scale_x, scale_y, 1.0),
        );

        // Gotta flip the image on the Y axis here
        // to account for OpenGL's origin being at the bottom-left.
        let new_param = flip_draw_param_vertical(param);

        self.image.draw_image_raw(ctx, new_param)
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

fn flip_draw_param_vertical(param: DrawParam) -> DrawParam {
    let param = if let Transform::Matrix(mat) = param.trans {
        param.transform(
            cgmath::Matrix4::from(mat)
                * cgmath::Matrix4::from_translation(cgmath::vec3(0.0, 1.0, 0.0))
                * cgmath::Matrix4::from_nonuniform_scale(1.0, -1.0, 1.0),
        )
    } else {
        panic!("Can not be called with a non-matrix DrawParam");
    };
    let new_src = Rect {
        x: param.src.x,
        y: (1.0 - param.src.h) - param.src.y,
        w: param.src.w,
        h: param.src.h,
    };
    param.src(new_src)
}

/// Set the `Canvas` to render to. Specifying `Option::None` will cause all
/// rendering to be done directly to the screen.
pub fn set_canvas(ctx: &mut Context, target: Option<&Canvas>) {
    ctx.gfx_context.canvas = target.cloned();
}
