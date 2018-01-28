//! A module that implements the [CIELAB color
//! space](https://en.wikipedia.org/wiki/Lab_color_space#CIELAB). The CIELAB color space is used as a
//! device-independent color space that has an L value for luminance and two opponent color axes for
//! chromaticity (loosely, hue). Formally, the three values that define a CIELAB color are called
//! L\*, A\*, and B\* to distinguish them from [generic
//! Lab](https://en.wikipedia.org/wiki/Lab_color_space), but for convenience they are just `L`, `a`,
//! and `b` in this module.

use color::{Color, XYZColor};



/// A color in the CIELAB color space.
pub struct CIELABColor {
    /// The luminance (loosely, brightness) of a given color. 0 is the lowest visible value and gives
    /// black, whereas 100 is the value of diffuse white: it is perhaps possible to have a higher
    /// value for reflective surfaces.
    L: f64;
    /// The first opponent color axis. By convention, this is usually between -128 and 127, with -128
    /// being fully green and 127 being fully magenta, but note that it is still possible to create
    /// "imaginary" colors (ones that cannot normally be seen by the human eye). Additionally,
    /// depending on the other two dimensions, many colors with a value in this range will still not
    /// be in the range of human vision.
    a: f64;
    /// The second opponent color axis. This is, like `a`, between -128 and 127 by convention for most
    /// visible colors, although it is possible to work with imaginary colors as well and many colors
    /// with a value in this range are not in the range of human vision. -128 is fully blue; 127 is
    /// fully yellow.
    b: f64;
}

impl Color for CIELABColor {
    fn from_xyz(xyz: XYZColor) -> CIELABColor {
        // https://en.wikipedia.org/wiki/Lab_color_space#CIELAB-CIEXYZ_conversions
        let f = |x| {
            let delta: f64 = 6.0 / 29.0;
            if x <= delta {
                x / (3 * delta * delta) + 4.0 / 29.0
            }
            else {
                x.pow(1.0 / 3.0)
            }
        }
        // now get the XYZ coordinates scaled from 0 to 1, using the illuminant's white point as (1,
        // 1, 1) and black as (0, 0, 0).
        let white_xyz = 
        let x_scaled = 
    }        
}

mod tests {
}
