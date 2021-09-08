use cgmath::{BaseFloat, Matrix3, Vector2};

pub trait Transform2d<S> {
    fn from_translation(v: Vector2<S>) -> Self;
    fn from_scale(value: S) -> Self;
    fn from_nonuniform_scale(x: S, y: S) -> Self;
}

impl<S: BaseFloat> Transform2d<S> for Matrix3<S> {
    /// Create a homogeneous transformation matrix from a translation vector.
    #[inline]
    #[rustfmt::skip]
    fn from_translation(v: Vector2<S>) -> Matrix3<S> {
        Matrix3::new(
            S::one(), S::zero(), S::zero(),
            S::zero(), S::one(), S::zero(),
            v.x, v.y, S::one(),
        )
    }

    /// Create a homogeneous transformation matrix from a scale value.
    #[inline]
    fn from_scale(value: S) -> Matrix3<S> {
        Matrix3::from_nonuniform_scale(value, value)
    }

    /// Create a homogeneous transformation matrix from a set of scale values.
    #[inline]
    #[rustfmt::skip]
    fn from_nonuniform_scale(x: S, y: S) -> Matrix3<S> {
        Matrix3::new(
            x, S::zero(), S::zero(),
            S::zero(), y, S::zero(),
            S::zero(), S::zero(), S::one(),
        )
    }
}
