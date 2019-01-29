/// An enum for specifying default and custom blend modes
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
    /// When combining two fragments, multiply their values together.
    Multiply,
    /// When combining two fragments, choose the source value
    Replace,
    /// When combining two fragments, choose the lighter value
    Lighten,
    /// When combining two fragments, choose the darker value
    Darken,
}
