use cgmath::Vector2;
use std::path;
use stdweb::web::html_element::ImageElement;

use crate::{
    error::{GameError, GameResult},
    filesystem,
    filesystem::File,
    graphics::{context::webgl::Texture, BlendMode, DrawParam, Drawable, Rect},
    Context,
};

#[derive(Clone, Debug)]
pub struct Image {
    texture: Option<Texture>,
    width: u16,
    height: u16,
}

impl Image {
    fn from_image_element(context: &mut Context, image_element: ImageElement) -> Image {
        Image {
            texture: Some(Texture::new(context, image_element.clone())),
            width: image_element.width() as u16,
            height: image_element.height() as u16,
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
            });
        }
        let texture = Some(Texture::from_rgba8(ctx, width, height, bytes));

        Ok(Image {
            texture,
            width,
            height,
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
}

impl Drawable for Image {
    fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult {
        if let Some(ref texture) = self.texture {
            ctx.gfx_context.webgl_context.draw_image(
                param.dest.into(),
                param.scale.into(),
                Vector2::new(self.width as f32, self.height as f32),
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
