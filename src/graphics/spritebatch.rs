use crate::{
    error::GameResult,
    graphics::{
        self, context::batch_shader, image::param_to_instance_transform, transform_rect, BlendMode,
        DrawParam, FilterMode, InstanceAttributes, Rect,
    },
    Context,
};

use std::cell::RefCell;

use cgmath::Vector4;
use miniquad::{Buffer, BufferType, PassAction};

#[derive(Debug)]
pub struct SpriteBatch {
    image: RefCell<graphics::Image>,
    sprites: Vec<DrawParam>,
    gpu_sprites: RefCell<Vec<InstanceAttributes>>,
    blend_mode: Option<BlendMode>,
}

/// An index of a particular sprite in a `SpriteBatch`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpriteIdx(usize);

impl SpriteBatch {
    /// Creates a new `SpriteBatch`, drawing with the given image.
    ///
    /// Takes ownership of the `Image`, but cloning an `Image` is
    /// cheap since they have an internal `Arc` containing the actual
    /// image data.
    pub fn new(image: graphics::Image) -> Self {
        Self {
            image: RefCell::new(image),
            sprites: vec![],
            blend_mode: None,
            gpu_sprites: RefCell::new(vec![]),
        }
    }

    /// Adds a new sprite to the sprite batch.
    ///
    /// Returns a handle with which type to modify the sprite using
    /// [`set()`](#method.set)
    pub fn add<P>(&mut self, param: P) -> SpriteIdx
    where
        P: Into<graphics::DrawParam>,
    {
        let param = param.into();
        self.sprites.push(param);
        SpriteIdx(self.sprites.len() - 1)
    }

    /// Removes all data from the sprite batch.
    pub fn clear(&mut self) {
        self.sprites.clear();
    }

    /// Unwraps and returns the contained `Image`
    pub fn into_inner(self) -> graphics::Image {
        self.image.into_inner()
    }

    /// Replaces the contained `Image`, returning the old one.
    pub fn set_image(&mut self, image: graphics::Image) -> graphics::Image {
        use std::mem;

        self.gpu_sprites = RefCell::new(vec![]);
        let mut self_image = self.image.borrow_mut();
        mem::replace(&mut *self_image, image)
    }

    /// Set the filter mode for the SpriteBatch.
    pub fn set_filter(&mut self, mode: FilterMode) {
        self.image.borrow_mut().set_filter(mode);
    }
}

impl graphics::Drawable for SpriteBatch {
    fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult {
        let mut image = self.image.borrow_mut();
        let mut gpu_sprites = self.gpu_sprites.borrow_mut();

        if self.sprites.len() > gpu_sprites.len() {
            gpu_sprites.resize(self.sprites.len(), InstanceAttributes::default());

            let buffer = Buffer::stream(
                &mut ctx.quad_ctx,
                BufferType::VertexBuffer,
                std::mem::size_of::<InstanceAttributes>() * self.sprites.len(),
            );

            if image.bindings.vertex_buffers.len() <= 1 {
                image.bindings.vertex_buffers.push(buffer);
            } else {
                image.bindings.vertex_buffers[1].delete();

                image.bindings.vertex_buffers[1] = buffer;
            }
        }

        for (n, param) in self.sprites.iter().enumerate() {
            let mut new_param = param.clone();
            let src_width = param.src.w;
            let src_height = param.src.h;
            let real_scale = graphics::Vector2::new(
                src_width * param.scale.x * f32::from(image.width),
                src_height * param.scale.y * f32::from(image.height),
            );
            new_param.scale = real_scale.into();

            let instance = InstanceAttributes {
                model: param_to_instance_transform(&new_param),
                source: Vector4::new(param.src.x, param.src.y, param.src.w, param.src.h),
                color: Vector4::new(param.color.r, param.color.g, param.color.b, param.color.a),
            };
            gpu_sprites[n] = instance;
        }

        image.bindings.vertex_buffers[1]
            .update(&mut ctx.quad_ctx, &gpu_sprites[0..self.sprites.len()]);

        let pass = ctx.framebuffer();
        ctx.quad_ctx.begin_pass(pass, PassAction::Nothing);
        ctx.quad_ctx
            .apply_pipeline(&ctx.gfx_context.sprite_pipeline);
        ctx.quad_ctx.apply_bindings(&image.bindings);

        let uniforms = batch_shader::Uniforms {
            projection: ctx.gfx_context.projection,
            model: param_to_instance_transform(&param),
        };
        ctx.quad_ctx.apply_uniforms(&uniforms);
        ctx.quad_ctx.draw(0, 6, self.sprites.len() as i32);

        ctx.quad_ctx.end_render_pass();

        Ok(())
    }

    fn dimensions(&self, _ctx: &mut Context) -> Option<Rect> {
        if self.sprites.is_empty() {
            return None;
        }
        let dimensions = self.image.borrow().dimensions();
        self.sprites
            .iter()
            .map(|&param| transform_rect(dimensions, param))
            .fold(None, |acc: Option<Rect>, rect| {
                Some(if let Some(acc) = acc {
                    acc.combine_with(rect)
                } else {
                    rect
                })
            })
    }

    fn set_blend_mode(&mut self, mode: Option<BlendMode>) {
        // TODO
        self.blend_mode = mode;
    }

    fn blend_mode(&self) -> Option<BlendMode> {
        self.blend_mode
    }
}
