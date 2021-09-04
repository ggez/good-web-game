use mint::{Point2, Vector2};

use crate::graphics::{Color, Rect};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DrawParam {
    /// A portion of the drawable to clip, as a fraction of the whole image.
    /// Defaults to the whole image `(0,0 to 1,1)` if omitted.
    pub src: Rect,
    /// The position to draw the graphic expressed as a `Point2`.
    pub dest: Point2<f32>,
    /// The orientation of the graphic in radians.
    pub rotation: f32,
    /// The x/y scale factors expressed as a `Vector2`.
    pub scale: Vector2<f32>,
    /// An offset from the center for transform operations like scale/rotation,
    /// with `0,0` meaning the origin and `1,1` meaning the opposite corner from the origin.
    /// By default these operations are done from the top-left corner, so to rotate something
    /// from the center specify `Point2::new(0.5, 0.5)` here.
    pub offset: Point2<f32>,
    /// A color to draw the target with.
    /// Default: white.
    pub color: Color,
}

impl Default for DrawParam {
    fn default() -> Self {
        DrawParam {
            src: Rect::one(),
            dest: [0.0, 0.0].into(),
            rotation: 0.0,
            scale: [1.0, 1.0].into(),
            offset: [0.0, 0.0].into(),
            color: Color::WHITE,
        }
    }
}

impl DrawParam {
    /// Create a new DrawParam with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the source rect
    pub fn src(mut self, src: Rect) -> Self {
        self.src = src;
        self
    }

    /// Set the dest point
    pub fn dest<P>(mut self, dest: P) -> Self
    where
        P: Into<mint::Point2<f32>>,
    {
        self.dest = dest.into();
        self
    }

    /// Set the drawable color.  This will be blended with whatever
    /// color the drawn object already is.
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set the rotation of the drawable.
    pub fn rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// Set the scaling factors of the drawable.
    pub fn scale<V>(mut self, scale: V) -> Self
    where
        V: Into<mint::Vector2<f32>>,
    {
        self.scale = scale.into();
        self
    }

    /// Set the transformation offset of the drawable.
    pub fn offset<P>(mut self, offset: P) -> Self
    where
        P: Into<mint::Point2<f32>>,
    {
        self.offset = offset.into();
        self
    }
}

/// Create a `DrawParam` from a location.
/// Note that this takes a single-element tuple.
/// It's a little weird but keeps the trait implementations
/// from clashing.
impl<P> From<(P,)> for DrawParam
where
    P: Into<mint::Point2<f32>>,
{
    fn from(location: (P,)) -> Self {
        DrawParam::new().dest(location.0)
    }
}

/// Create a `DrawParam` from a location and color
impl<P> From<(P, Color)> for DrawParam
where
    P: Into<mint::Point2<f32>>,
{
    fn from((location, color): (P, Color)) -> Self {
        DrawParam::new().dest(location).color(color)
    }
}

/// Create a `DrawParam` from a location, rotation and color
impl<P> From<(P, f32, Color)> for DrawParam
where
    P: Into<mint::Point2<f32>>,
{
    fn from((location, rotation, color): (P, f32, Color)) -> Self {
        DrawParam::new()
            .dest(location)
            .rotation(rotation)
            .color(color)
    }
}

/// Create a `DrawParam` from a location, rotation, offset and color
impl<P> From<(P, f32, P, Color)> for DrawParam
where
    P: Into<mint::Point2<f32>>,
{
    fn from((location, rotation, offset, color): (P, f32, P, Color)) -> Self {
        DrawParam::new()
            .dest(location)
            .rotation(rotation)
            .offset(offset)
            .color(color)
    }
}

/// Create a `DrawParam` from a location, rotation, offset, scale and color
impl<P, V> From<(P, f32, P, V, Color)> for DrawParam
where
    P: Into<mint::Point2<f32>>,
    V: Into<mint::Vector2<f32>>,
{
    fn from((location, rotation, offset, scale, color): (P, f32, P, V, Color)) -> Self {
        DrawParam::new()
            .dest(location)
            .rotation(rotation)
            .offset(offset)
            .scale(scale)
            .color(color)
    }
}
