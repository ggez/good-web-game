use crate::{
    error::GameResult,
    graphics::{
        self, apply_uniforms, transform_rect, BlendMode, DrawParam, FilterMode, InstanceAttributes,
        Rect,
    },
    Context,
};

use std::cell::RefCell;

use miniquad::{Buffer, BufferType, PassAction};

#[derive(Debug)]
pub struct SpriteBatch {
    image: RefCell<graphics::Image>,
    sprites: Vec<DrawParam>,
    gpu_sprites: RefCell<Vec<InstanceAttributes>>,
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

    /// Alters a sprite in the batch to use the given draw params
    pub fn set<P>(&mut self, handle: SpriteIdx, param: P) -> GameResult
    where
        P: Into<graphics::DrawParam>,
    {
        if handle.0 < self.sprites.len() {
            self.sprites[handle.0] = param.into();
            Ok(())
        } else {
            Err(crate::error::GameError::RenderError(String::from(
                "Provided index is out of bounds.",
            )))
        }
    }

    /// Returns a reference to the sprites.
    pub fn get_sprites(&self) -> &[DrawParam] {
        &self.sprites
    }

    /// Returns a mutable reference to the sprites.
    ///
    /// Unlike with `MeshBatch`, manually calling `flush` after altering sprites
    /// in this slice is currently unnecessary, as `SpriteBatch` flushes automatically
    /// on every draw call. This might change in the future though.
    pub fn get_sprites_mut(&mut self) -> &mut [DrawParam] {
        &mut self.sprites
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
    fn draw(
        &self,
        ctx: &mut Context,
        quad_ctx: &mut miniquad::graphics::GraphicsContext,
        param: DrawParam,
    ) -> GameResult {
        {
            let mut image = self.image.borrow_mut();
            let mut gpu_sprites = self.gpu_sprites.borrow_mut();

            if self.sprites.len() > gpu_sprites.len() {
                gpu_sprites.resize(self.sprites.len(), InstanceAttributes::default());

                let buffer = Buffer::stream(
                    quad_ctx,
                    BufferType::VertexBuffer,
                    std::mem::size_of::<InstanceAttributes>() * self.sprites.len(),
                );

                image.bindings.vertex_buffers[1].delete();
                image.bindings.vertex_buffers[1] = buffer;
            }

            for (n, param) in self.sprites.iter().enumerate() {
                let mut new_param = *param;
                let src_width = param.src.w;
                let src_height = param.src.h;
                // We have to mess with the scale to make everything
                // be its-unit-size-in-pixels.
                let scale_x = src_width * f32::from(image.width);
                let scale_y = src_height * f32::from(image.height);

                use crate::graphics::Transform;
                new_param = match new_param.trans {
                    Transform::Values { scale, .. } => new_param.scale(mint::Vector2 {
                        x: scale.x * scale_x,
                        y: scale.y * scale_y,
                    }),
                    Transform::Matrix(m) => new_param.transform(
                        cgmath::Matrix4::from(m)
                            * cgmath::Matrix4::from_nonuniform_scale(scale_x, scale_y, 1.0),
                    ),
                };

                let instance = InstanceAttributes::from(&new_param);
                gpu_sprites[n] = instance;
            }

            image.bindings.vertex_buffers[1].update(quad_ctx, &gpu_sprites[0..self.sprites.len()]);

            let pass = ctx.framebuffer();
            quad_ctx.begin_pass(pass, PassAction::Nothing);
            quad_ctx.apply_bindings(&image.bindings);
        }
        let shader_id = *ctx.gfx_context.current_shader.borrow();
        let current_shader = &mut ctx.gfx_context.shaders[shader_id];
        quad_ctx.apply_pipeline(&current_shader.pipeline);

        apply_uniforms(
            ctx,
            quad_ctx,
            shader_id,
            Some(cgmath::Matrix4::from(param.trans.to_bare_matrix())),
        );

        let mut custom_blend = false;
        if let Some(blend_mode) = self.blend_mode() {
            custom_blend = true;
            crate::graphics::set_current_blend_mode(quad_ctx, blend_mode)
        }

        quad_ctx.draw(0, 6, self.sprites.len() as i32);

        // restore default blend mode
        if custom_blend {
            crate::graphics::restore_blend_mode(ctx, quad_ctx);
        }

        quad_ctx.end_render_pass();

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
        self.image.get_mut().set_blend_mode(mode);
    }

    fn blend_mode(&self) -> Option<BlendMode> {
        self.image.borrow().blend_mode()
    }
}
