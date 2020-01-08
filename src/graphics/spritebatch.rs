use crate::{
    error::GameResult,
    graphics::{
        self,
        image::{batch_shader, param_to_instance_transform},
        transform_rect, BlendMode, DrawParam, InstanceAttributes, Rect,
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
            gpu_sprites: RefCell::new(vec![InstanceAttributes::default()]),
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
}

impl graphics::Drawable for SpriteBatch {
    fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult {
        let mut image = self.image.borrow_mut();
        let mut gpu_sprites = self.gpu_sprites.borrow_mut();

        if self.sprites.len() != gpu_sprites.len() {
            gpu_sprites.resize(self.sprites.len(), InstanceAttributes::default());

            image.bindings.vertex_buffers[1] = Buffer::stream(
                &mut ctx.quad_ctx,
                BufferType::VertexBuffer,
                std::mem::size_of::<InstanceAttributes>() * gpu_sprites.len(),
            );
        }
        // // TODO: This is really nasty and doesn't really do the batching
        for (n, sprite_param) in self.sprites.iter().enumerate() {
            let mut new_param = sprite_param.clone();

            new_param.dest.x = new_param.dest.x * param.scale.x + param.dest.x;
            new_param.dest.y = new_param.dest.y * param.scale.y + param.dest.y;
            new_param.scale.x *= param.scale.x;
            new_param.scale.y *= param.scale.y;

            let instance = InstanceAttributes {
                model: param_to_instance_transform(&new_param, image.width, image.height),
                source: Vector4::new(
                    new_param.src.x,
                    new_param.src.y,
                    new_param.src.w,
                    new_param.src.h,
                ),
                color: Vector4::new(
                    new_param.color.r,
                    new_param.color.g,
                    new_param.color.b,
                    new_param.color.a,
                ),
            };
            gpu_sprites[n] = instance;
        }

        image.bindings.vertex_buffers[1].update(ctx.quad_ctx, &*gpu_sprites);

        let pass = ctx.framebuffer();
        ctx.quad_ctx.begin_pass(pass, PassAction::Nothing);
        ctx.quad_ctx.apply_pipeline(&image.pipeline);
        ctx.quad_ctx.apply_bindings(&image.bindings);

        let uniforms = batch_shader::Uniforms {
            projection: ctx.internal.gfx_context.projection,
        };
        ctx.quad_ctx.apply_uniforms(&uniforms);
        ctx.quad_ctx.draw(0, 6, gpu_sprites.len() as i32);

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
