use cgmath::{Point2, Vector2, Vector4};
use std::{path, cell::Cell};
use stdweb::web::html_element::ImageElement;

use crate::{
    error::{GameError, GameResult},
    filesystem,
    filesystem::File,
    graphics::{context::webgl::{Texture, NEAREST_FILTER, LINEAR_FILTER}, BlendMode, DrawParam, Drawable, Rect},
    Context,
};

#[derive(Clone, Debug)]
pub struct Image {
    texture: Option<Texture>,
    width: u16,
    height: u16,
    filter: FilterMode,
    dirty_filter: Cell<bool>,
}

#[derive(Clone, Copy, Debug)]
pub enum FilterMode {
    Linear = LINEAR_FILTER as isize,
    Nearest = NEAREST_FILTER as isize,
}

impl Image {
    fn from_image_element(context: &mut Context, image_element: ImageElement) -> Image {
        Image {
            texture: Some(Texture::new(context, image_element.clone())),
            width: image_element.width() as u16,
            height: image_element.height() as u16,
            filter: FilterMode::Linear,
            dirty_filter: Cell::new(false),
        }
    }

    pub fn new<P: AsRef<path::Path>>(ctx: &mut Context, path: P) -> GameResult<Self> {
        let file = filesystem::open(ctx, path)?;

        match file {
            File::Image(image_element) => Ok(Image::from_image_element(ctx, image_element.clone())),
            _ => Err(GameError::UnknownError("File is not an image!".to_string())),
        }
    }

    pub fn from_rgba8(
        ctx: &mut crate::Context,
        width: u16,
        height: u16,
        bytes: &[u8],
    ) -> GameResult<Image> {
        if width == 0 || height == 0 {
            return Ok(Image {
                texture: None,
                width: width,
                height: height,
                filter: FilterMode::Linear,
                dirty_filter: Cell::new(false),
            });
        }
        let texture = Some(Texture::from_rgba8(ctx, width, height, bytes));

        Ok(Image {
            texture,
            width,
            height,
            filter: FilterMode::Linear,
            dirty_filter: Cell::new(false),
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
        if let Some(ref texture) = self.texture {
            if self.dirty_filter.get() {
                self.dirty_filter.set(false);

                texture.set_filter(ctx, self.filter as i32);
            }

            let real_size = Vector2::new(param.src.w * self.width as f32, param.src.h * self.height as f32);

            ctx.gfx_context.webgl_context.draw_image(
                Point2::new(
                    param.dest.x - real_size.x * param.offset.x * param.scale.x,
                    param.dest.y - real_size.y * param.offset.y * param.scale.y,
                ),
                Vector2::new(real_size.x * param.scale.x, real_size.y * param.scale.y),
                Vector4::new(param.src.x, param.src.y, param.src.w, param.src.h),
                param.color,
                texture,
            );
        }

        // let self_width = self.image_element.width() as f32;
        // let self_height = self.image_element.height() as f32;
        // let src_x = param.src.x * self_width;
        // let src_y = param.src.y * self_height;
        // let src_width = param.src.w * self_width;
        // let src_height = param.src.h * self_height;
        // let dst_width = (self_width * param.scale.x) as f32;
        // let dst_height = (self_height * param.scale.y) as f32;
        // let mut dst_x = param.dest.x;
        // let mut dst_y = param.dest.y;

        // ctx.canvas_context.canvas.save();

        // if param.rotation != 0. {
        //     let rotation = cgmath::Matrix3::from_angle_z(cgmath::Rad(param.rotation));
        //     let offset_translate = Matrix3::from_translation(Vector2::new(
        //         -dst_width * param.offset.x,
        //         -dst_height * param.offset.y,
        //     ));
        //     let offset_translate_inv = Matrix3::from_translation(Vector2::new(
        //         dst_width * param.offset.x,
        //         dst_height * param.offset.y,
        //     ));
        //     let translate = Matrix3::from_translation(param.dest.to_vec());
        //     let transform = translate * offset_translate_inv * rotation * offset_translate;
        //     ctx.canvas_context.transform_with_matrix(&transform);

        //     dst_x = 0.;
        //     dst_y = 0.;
        // }

        // if param.color != WHITE {
        //     js!(@{&ctx.canvas_context
        //         .canvas}.filter = @{format!("url({})", filter_hack_url(param.color))});
        // }

        // ctx.canvas_context
        //     .canvas
        //     .draw_image_s(
        //         self.image_element.clone(),
        //         src_x as f64,
        //         src_y as f64,
        //         src_width as f64,
        //         src_height as f64,
        //         dst_x as f64,
        //         dst_y as f64,
        //         dst_width as f64,
        //         dst_height as f64,
        //     )
        //     .unwrap();

        // ctx.canvas_context.canvas.restore();

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
