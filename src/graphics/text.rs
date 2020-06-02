use super::{BlendMode, Color, DrawParam, Drawable, GameResult, Rect};

use crate::{filesystem, graphics::param_to_instance_transform};

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

        let font = ctx
            .gfx_context
            .load_font(&mut ctx.quad_ctx, &bytes[..], 50)?;
        Ok(Font(FontId(font)))
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct Scale {
    /// Horizontal scale, in pixels.
    pub x: f32,
    /// Vertical scale, in pixels.
    pub y: f32,
}

impl Scale {
    /// Uniform scaling, equivalent to `Scale { x: s, y: s }`.
    #[inline]
    pub fn uniform(s: f32) -> Scale {
        Scale { x: s, y: s }
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
    pub scale: Option<Scale>,
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
    pub fn scale(mut self, scale: Scale) -> TextFragment {
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
        text.into().scale(Scale::uniform(scale))
    }
}

impl<T> From<(T, Font, f32)> for TextFragment
where
    T: Into<TextFragment>,
{
    fn from((text, font, scale): (T, Font, f32)) -> TextFragment {
        text.into().font(font).scale(Scale::uniform(scale))
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
        let scale = self.fragment.scale.unwrap_or(Scale { x: 1., y: 1. });

        (text.get_width() * scale.x, scale.y)
    }
}

impl Drawable for Text {
    fn draw(&self, ctx: &mut crate::Context, param: DrawParam) -> GameResult {
        let text = self.lazy_init_gpu_text(ctx);

        let scale = self.fragment.scale.unwrap_or(Scale { x: 1., y: 1. });

        let mut new_param = param;
        new_param.scale =
            cgmath::Vector2::new(scale.x * param.scale.x * 1., -scale.y * param.scale.y * 1.)
                .into();
        // 0.7 comes from usual difference between ascender line and cap line, whatever it means
        new_param.dest.y += scale.y * param.scale.y * 0.7;

        let transform = param_to_instance_transform(&new_param);
        let projection = ctx.gfx_context.projection;

        let mvp = projection * transform;

        miniquad_text_rusttype::draw(
            &mut ctx.quad_ctx,
            &text,
            &ctx.gfx_context.text_system,
            mvp,
            (param.color.r, param.color.g, param.color.b, param.color.a),
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
