//use miniquad::{Equation, BlendValue, BlendFactor};

//type Blend = Option<(Equation, BlendFactor, BlendFactor)>;

use miniquad::{BlendFactor, BlendState, BlendValue, Equation};

/// An enum for specifying default and custom blend modes
///
/// If you want to know what these actually do take a look at the implementation of `From<BlendMode> for Blend`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlendMode {
    /// When combining two fragments, add their values together, saturating
    /// at 1.0
    Add,
    /// When combining two fragments, subtract the source value from the
    /// destination value
    Subtract,
    /// When combining two fragments, add the value of the source times its
    /// alpha channel with the value of the destination multiplied by the inverse
    /// of the source alpha channel. Has the usual transparency effect: mixes the
    /// two colors using a fraction of each one specified by the alpha of the source.
    Alpha,
    /// When combining two fragments, multiply their values together (including alpha)
    Multiply,
    /// When combining two fragments, choose the source value (including source alpha)
    Replace,
    /// When using premultiplied alpha, use this.
    ///
    /// You usually want to use this blend mode for drawing canvases
    /// containing semi-transparent imagery.
    /// For an explanation on this see: `<https://github.com/ggez/ggez/issues/694#issuecomment-853724926>`
    Premultiplied,
}

impl From<BlendMode> for (BlendState, BlendState) {
    fn from(bm: BlendMode) -> Self {
        match bm {
            BlendMode::Add => (
                BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::One,
                ),
                BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::One,
                ),
            ),
            BlendMode::Subtract => (
                BlendState::new(
                    Equation::ReverseSubtract,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::One,
                ),
                BlendState::new(Equation::Add, BlendFactor::Zero, BlendFactor::One),
            ),
            BlendMode::Alpha => (
                BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                ),
                BlendState::new(
                    Equation::Add,
                    BlendFactor::OneMinusValue(BlendValue::DestinationAlpha),
                    BlendFactor::One,
                ),
            ),
            BlendMode::Premultiplied => (
                BlendState::new(
                    Equation::Add,
                    BlendFactor::One,
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                ),
                BlendState::new(
                    Equation::Add,
                    BlendFactor::OneMinusValue(BlendValue::DestinationAlpha),
                    BlendFactor::One,
                ),
            ),
            BlendMode::Multiply => (
                BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::DestinationColor),
                    BlendFactor::Zero,
                ),
                BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::DestinationAlpha),
                    BlendFactor::Zero,
                ),
            ),
            BlendMode::Replace => (
                BlendState::new(Equation::Add, BlendFactor::One, BlendFactor::Zero),
                BlendState::new(Equation::Add, BlendFactor::One, BlendFactor::Zero),
            ),
        }
    }
}
