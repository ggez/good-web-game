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
    /// Create a new `Canvas` with the specified size.
    pub fn new(
        ctx: &mut Context,
        quad_ctx: &mut miniquad::graphics::GraphicsContext,
        width: u16,
        height: u16,
    ) -> GameResult<Canvas> {
        let texture = Texture::new_render_texture(
            quad_ctx,
            TextureParams {
                width: width as u32,
                height: height as u32,
                format: TextureFormat::RGBA8,
                ..Default::default()
            },
        );

        let image = Image::from_texture(quad_ctx, texture, ctx.gfx_context.default_filter)?;

        let offscreen_pass = RenderPass::new(quad_ctx, texture, None);

        Ok(Canvas {
            image,
            offscreen_pass,
        })
    }

    /// Create a new `Canvas` with the current window dimensions.
    pub fn with_window_size(
        ctx: &mut Context,
        quad_ctx: &mut miniquad::graphics::GraphicsContext,
    ) -> GameResult<Canvas> {
        use crate::graphics;
        let (w, h) = graphics::drawable_size(quad_ctx);
        // Default to no multisampling
        Canvas::new(ctx, quad_ctx, w as u16, h as u16)
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
    fn draw(
        &self,
        ctx: &mut Context,
        quad_ctx: &mut miniquad::graphics::GraphicsContext,
        param: DrawParam,
    ) -> GameResult {
        // We have to mess with the scale to make everything
        // be its-unit-size-in-pixels.
        let scale_x = param.src.w * f32::from(self.width());
        let scale_y = param.src.h * f32::from(self.height());

        let scaled_param = match param.trans {
            Transform::Values { scale, .. } => {
                let s_param = param.scale(mint::Vector2 {
                    x: scale.x * scale_x,
                    y: scale.y * scale_y,
                });
                s_param.transform(s_param.trans.to_bare_matrix())
            }
            Transform::Matrix(m) => param.transform(
                cgmath::Matrix4::from(m)
                    * cgmath::Matrix4::from_nonuniform_scale(scale_x, scale_y, 1.0),
            ),
        };

        // Gotta flip the image on the Y axis here
        // to account for OpenGL's origin being at the bottom-left.
        let new_param = flip_draw_param_vertical(scaled_param);

        self.image.draw_image_raw(ctx, quad_ctx, new_param)
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

#[rustfmt::skip]
/// equal to
///
/// Matrix4::from_translation(cgmath::vec3(0.0, 1.0, 0.0)) * Matrix4::from_nonuniform_scale(1.0, -1.0, 1.0),
const FLIP_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, -1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 1.0, 0.0, 1.0,
);

fn flip_draw_param_vertical(param: DrawParam) -> DrawParam {
    let param = if let Transform::Matrix(mat) = param.trans {
        param.transform(cgmath::Matrix4::from(mat) * FLIP_MATRIX)
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
