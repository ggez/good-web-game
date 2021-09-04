//use miniquad::{Equation, BlendValue, BlendFactor};

//type Blend = Option<(Equation, BlendFactor, BlendFactor)>;

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
    /// When combining two fragments, subtract the destination color from a constant
    /// color using the source color as weight. Has an invert effect with the constant
    /// color as base and source color controlling displacement from the base color.
    /// A white source color and a white value results in plain invert. The output
    /// alpha is same as destination alpha.
    Invert,
    /// When combining two fragments, multiply their values together (including alpha)
    Multiply,
    /// When combining two fragments, choose the source value (including source alpha)
    Replace,
    /// When combining two fragments, choose the lighter value
    Lighten,
    /// When combining two fragments, choose the darker value
    Darken,
    /// When using premultiplied alpha, use this.
    ///
    /// You usually want to use this blend mode for drawing canvases
    /// containing semi-transparent imagery.
    /// For an explanation on this see: https://github.com/ggez/ggez/issues/694#issuecomment-853724926
    Premultiplied,
}

// TODO: find out if there might exist a way to get alpha blending using miniquad;
//       if not, at least create some color-only blend modes
/*
impl From<BlendMode> for Blend {
    fn from(bm: BlendMode) -> Self {
        match bm {
            BlendMode::Add => Blend {
                color: BlendChannel {
                    equation: Equation::Add,
                    source: BlendFactor::Value(BlendValue::SourceAlpha),
                    destination: BlendFactor::One,
                },
                alpha: BlendChannel {
                    equation: Equation::Add,
                    source: BlendFactor::OneMinusValue(BlendValue::DestinationAlpha),
                    destination: BlendFactor::One,
                },
            },
            BlendMode::Subtract => Blend {
                color: BlendChannel {
                    equation: Equation::ReverseSubtract,
                    source: BlendFactor::Value(BlendValue::SourceAlpha),
                    destination: BlendFactor::One,
                },
                alpha: BlendChannel {
                    equation: Equation::Add,
                    source: BlendFactor::Zero,
                    destination: BlendFactor::One,
                },
            },
            BlendMode::Alpha => Blend {
                color: BlendChannel {
                    equation: Equation::Add,
                    source: BlendFactor::Value(BlendValue::SourceAlpha),
                    destination: BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                },
                alpha: BlendChannel {
                    equation: Equation::Add,
                    source: BlendFactor::OneMinusValue(BlendValue::DestinationAlpha),
                    destination: BlendFactor::One,
                },
            },
            BlendMode::Invert => blend::INVERT,
            BlendMode::Multiply => blend::MULTIPLY,
            BlendMode::Replace => blend::REPLACE,
            BlendMode::Lighten => Blend {
                color: BlendChannel {
                    equation: Equation::Max,
                    source: BlendFactor::Value(BlendValue::SourceAlpha),
                    destination: BlendFactor::One,
                },
                alpha: BlendChannel {
                    equation: Equation::Add,
                    source: BlendFactor::OneMinusValue(BlendValue::DestinationAlpha),
                    destination: BlendFactor::One,
                },
            },
            BlendMode::Darken => Blend {
                color: BlendChannel {
                    equation: Equation::Min,
                    source: BlendFactor::Value(BlendValue::SourceAlpha),
                    destination: BlendFactor::One,
                },
                alpha: BlendChannel {
                    equation: Equation::Add,
                    source: BlendFactor::OneMinusValue(BlendValue::DestinationAlpha),
                    destination: BlendFactor::One,
                },
            },
            BlendMode::Premultiplied => Blend {
                color: BlendChannel {
                    equation: Equation::Add,
                    source: BlendFactor::One,
                    destination: BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                },
                alpha: BlendChannel {
                    equation: Equation::Add,
                    source: BlendFactor::OneMinusValue(BlendValue::DestinationAlpha),
                    destination: BlendFactor::One,
                },
            },
        }
    }
}
*/
