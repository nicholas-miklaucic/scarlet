//! This module implements the CIELCHuv color space, a cylindrical transformation of the
//! CIELUV space, akin to the relationship between CIELAB and CIEHCL without the uv.

use color::{Color, XYZColor};
use coord::Coord;
use illuminants::Illuminant;

#[derive(Debug, Copy, Clone)]
pub struct CIELCHuvColor {
    /// The luminance component. Exactly the same as CIELAB, CIELUV, and CIELCH. Varies between 0 and
    /// 100 by definition.
    pub l: f64,
    /// The chroma component: essentially, how colorful the color is compared to white. (This is
    /// contrasted with saturation, which is how colorful a color is when compared to an equivalently
    /// bright grayscale color: a dark, deep red may have high saturation and low chroma.) This varies
    /// between 0 and about 141 for most visible colors, and is the radius in cylindrical coordinates.
    pub c: f64,
    /// The hue component: essentially, what wavelengths of light have the highest reflectance. This
    /// is the angle from the vertical axis in cylindrical coordinates. 0 degrees corresponds to red,
    /// 90 to yellow, 180 to green, and 270 to blue. (These are called *unique hues.*) It ranges from
    /// 0 to 360, and any value outside that range will be interpreted as its value if one added or
    /// subtracted multiples of 360 to bring the value inside that range.
    pub h: f64,
}
