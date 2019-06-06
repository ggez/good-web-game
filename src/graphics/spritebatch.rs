use crate::{
    error::GameResult,
    graphics::{self, transform_rect, BlendMode, Rect, DrawParam},
    Context,
};

#[derive(Debug, Clone)]
pub struct SpriteBatch {
    image: graphics::Image,
    sprites: Vec<graphics::DrawParam>,
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
            image,
            sprites: vec![],
            blend_mode: None,
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
        self.sprites.push(param.into());
        SpriteIdx(self.sprites.len() - 1)
    }

    /// Removes all data from the sprite batch.
    pub fn clear(&mut self) {
        self.sprites.clear();
    }

    /// Unwraps and returns the contained `Image`
    pub fn into_inner(self) -> graphics::Image {
        self.image
    }
}

impl graphics::Drawable for SpriteBatch {
    fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult {
        // TODO: This is really nasty and doesn't really do the batching
        for &sprite_param in self.sprites.iter() {
            let mut new_param = sprite_param.clone();

            new_param.dest.x = new_param.dest.x * param.scale.x + param.dest.x;
            new_param.dest.y = new_param.dest.y * param.scale.y + param.dest.y;
            new_param.scale.x *= param.scale.x;
            new_param.scale.y *= param.scale.y;

            graphics::draw(ctx, &self.image, new_param)?;
        }

        Ok(())
    }

    fn dimensions(&self, _ctx: &mut Context) -> Option<Rect> {
        if self.sprites.is_empty() {
            return None;
        }
        let dimensions = self.image.dimensions();
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
