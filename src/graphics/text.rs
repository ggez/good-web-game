use super::{BlendMode, Color, DrawParam, Drawable, GameResult, Rect};

use crate::filesystem;

use miniquad_text_rusttype::{FontTexture, TextDisplay};

use std::{cell::Ref, ops::Deref, path, rc::Rc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Font(pub FontId);

impl Default for Font {
    fn default() -> Font {
        Font(FontId(0))
    }
}

impl Font {
    /// Should construct font from the ttf file path.
    pub fn new<P: AsRef<path::Path>>(
        ctx: &mut crate::Context,
        ttf_filepath: P,
    ) -> GameResult<Font> {
        use std::io::Read;

        let mut file = filesystem::open(ctx, ttf_filepath)?;

        let mut bytes = vec![];
        file.bytes.read_to_end(&mut bytes)?;

        let font =
            ctx.gfx_context
                .load_font(&mut ctx.quad_ctx, &bytes[..], ctx.gfx_context.font_size)?;
        Ok(Font(FontId(font)))
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct PxScale {
    /// Horizontal scale, in pixels.
    pub x: f32,
    /// Vertical scale, in pixels.
    pub y: f32,
}

impl PxScale {
    /// Uniform scaling, equivalent to `Scale { x: s, y: s }`.
    #[inline]
    pub fn uniform(s: f32) -> PxScale {
        PxScale { x: s, y: s }
    }
}

impl From<f32> for PxScale {
    fn from(float: f32) -> Self {
        Self::uniform(float)
    }
}

/// A piece of text with optional color, font and font scale information.
/// Drawing text generally involves one or more of these.
/// These options take precedence over any similar field/argument.
/// Implements `From` for `char`, `&str`, `String` and
/// `(String, Font, Scale)`.
#[derive(Clone, Debug)]
pub struct TextFragment {
    /// Text string itself.
    pub text: String,
    /// Fragment's color, defaults to text's color.
    pub color: Option<Color>,
    /// Fragment's font, defaults to text's font.
    pub font: Option<Font>,
    /// Fragment's scale, defaults to text's scale.
    pub scale: Option<PxScale>,
}

impl Default for TextFragment {
    fn default() -> Self {
        TextFragment {
            text: "".into(),
            color: None,
            font: None,
            scale: None,
        }
    }
}

impl TextFragment {
    /// Creates a new fragment from `String` or `&str`.
    pub fn new<T: Into<Self>>(text: T) -> Self {
        text.into()
    }

    /// Set fragment's color, overrides text's color.
    pub fn color(mut self, color: Color) -> TextFragment {
        self.color = Some(color);
        self
    }

    /// Set fragment's font, overrides text's font.
    pub fn font(mut self, font: Font) -> TextFragment {
        self.font = Some(font);
        self
    }

    /// Set fragment's scale, overrides text's scale.
    pub fn scale(mut self, scale: PxScale) -> TextFragment {
        self.scale = Some(scale);
        self
    }
}

impl<'a> From<&'a str> for TextFragment {
    fn from(text: &'a str) -> TextFragment {
        TextFragment {
            text: text.to_owned(),
            ..Default::default()
        }
    }
}

impl From<char> for TextFragment {
    fn from(ch: char) -> TextFragment {
        TextFragment {
            text: ch.to_string(),
            ..Default::default()
        }
    }
}

impl From<String> for TextFragment {
    fn from(text: String) -> TextFragment {
        TextFragment {
            text,
            ..Default::default()
        }
    }
}

impl<T> From<(T, f32)> for TextFragment
where
    T: Into<TextFragment>,
{
    fn from((text, scale): (T, f32)) -> TextFragment {
        text.into().scale(PxScale::uniform(scale))
    }
}

impl<T> From<(T, Font, f32)> for TextFragment
where
    T: Into<TextFragment>,
{
    fn from((text, font, scale): (T, Font, f32)) -> TextFragment {
        text.into().font(font).scale(PxScale::uniform(scale))
    }
}

pub struct Text {
    fragment: TextFragment,
    font_id: FontId,
    gpu_text: std::cell::RefCell<Option<TextDisplay<std::rc::Rc<FontTexture>>>>,
}

impl Text {
    /// Creates a `Text` from a `TextFragment`.
    ///
    /// ```rust
    /// # use ggez::graphics::Text;
    /// # fn main() {
    /// let text = Text::new("foo");
    /// # }
    /// ```
    pub fn new<F>(fragment: F) -> Text
    where
        F: Into<TextFragment>,
    {
        Text {
            fragment: fragment.into(),
            font_id: FontId(0),
            gpu_text: std::cell::RefCell::new(None),
        }
    }

    fn lazy_init_gpu_text<'a>(
        &'a self,
        ctx: &mut crate::Context,
    ) -> impl Deref<Target = TextDisplay<Rc<FontTexture>>> + 'a {
        let font =
            ctx.gfx_context.fonts_cache[self.fragment.font.map_or(self.font_id, |f| f.0).0].clone();
        if self.gpu_text.borrow().is_none() {
            let text = miniquad_text_rusttype::TextDisplay::new(
                &mut ctx.quad_ctx,
                &ctx.gfx_context.text_system,
                font,
                &self.fragment.text,
            );

            *self.gpu_text.borrow_mut() = Some(text);
        }

        Ref::map(self.gpu_text.borrow(), |t| t.as_ref().unwrap())
    }
    pub fn dimensions(&self, ctx: &mut crate::Context) -> (f32, f32) {
        let text = self.lazy_init_gpu_text(ctx);
        let scale = self.fragment.scale.unwrap_or(PxScale { x: 1., y: 1. });

        (text.get_width() * scale.x, scale.y)
    }
}

impl Drawable for Text {
    fn draw(&self, ctx: &mut crate::Context, param: DrawParam) -> GameResult {
        let text = self.lazy_init_gpu_text(ctx);

        let frag_scale = self.fragment.scale.unwrap_or(PxScale { x: 1., y: 1. });

        use crate::graphics::Transform;
        let new_param = match param.trans {
            Transform::Values { scale, dest, .. } => param
                .scale(mint::Vector2 {
                    x: scale.x * frag_scale.x,
                    y: -scale.y * frag_scale.y,
                })
                .dest(mint::Vector2 {
                    // 0.7 comes from usual difference between ascender line and cap line, whatever it means
                    x: dest.x,
                    y: dest.y + frag_scale.y * scale.y * 0.7,
                }),
            Transform::Matrix(m) => param.transform(
                cgmath::Matrix4::from(m)
                    * cgmath::Matrix4::from_nonuniform_scale(frag_scale.x, frag_scale.y, 1.0),
            ),
        };

        let transform: cgmath::Matrix4<f32> = new_param.trans.to_bare_matrix().into();
        let projection = ctx.gfx_context.projection;

        let mvp = projection * transform;

        // if there's a specified color for the fragment use that
        // if not, use the color of the DrawParam
        // TODO: maybe instead recreate the expected color blending effect here
        let color =
        if let Some(frag_color) = self.fragment.color {
            frag_color
        } else {
            param.color
        };

        miniquad_text_rusttype::draw(
            &mut ctx.quad_ctx,
            &text,
            &ctx.gfx_context.text_system,
            mvp,
            (color.r, color.g, color.b, color.a),
        );
        Ok(())
    }

    fn dimensions(&self, ctx: &mut crate::Context) -> Option<Rect> {
        let (w, h) = self.dimensions(ctx);

        Some(Rect::new(0., 0., w as f32, h as f32))
    }

    fn set_blend_mode(&mut self, _: Option<BlendMode>) {
        unimplemented!()
    }

    fn blend_mode(&self) -> Option<BlendMode> {
        unimplemented!()
    }
}

impl Drop for Text {
    fn drop(&mut self) {
        let t_display = std::mem::replace(self.gpu_text.get_mut(), None);
        if let Some(text_display) = t_display {
            crate::graphics::add_dropped_text(text_display);
        }
    }
}
