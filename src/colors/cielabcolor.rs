//! A module that implements the [CIELAB color
//! space](https://en.wikipedia.org/wiki/Lab_color_space#CIELAB). The CIELAB color space is used as a
//! device-independent color space that has an L value for luminance and two opponent color axes for
//! chromaticity (loosely, hue). Formally, the three values that define a CIELAB color are called
//! L\*, A\*, and B\* to distinguish them from [generic
//! Lab](https://en.wikipedia.org/wiki/Lab_color_space), but for convenience they are just `L`, `a`,
//! and `b` in this module.

use color::{Color, XYZColor};
use coord::Coord;
use illuminants::Illuminant;

/// A color in the CIELAB color space.
/// # Example
/// Unlike spaces such as HSV and RGB, moving a and b linearly will create roughly smooth change in
/// color.
///
/// ```
/// # use scarlet::prelude::*;
/// # use scarlet::colors::CIELABColor;
/// // roughly blue-green
/// let mut color = CIELABColor{l: 50., a: -100., b: -100.};
/// for _i in 0..10 {
///     // make warmer: positive a and b direction
///     color.a = color.a + 20.;
///     color.b = color.b + 20.;
///     println!("{}", color.convert::<RGBColor>().to_string());
/// }
/// // prints the following:
/// // #0098FF
/// // #0092DD
/// // #008ABA
/// // #2F8298
/// // #777777
/// // #9E6956
/// // #BD5735
/// // #D73C0A
/// // #F00000
/// // #FF0000
/// // note that the end might have been truncated to fit in sRGB's gamut on either side
/// ```
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CIELABColor {
    /// The luminance (loosely, brightness) of a given color. 0 is the lowest visible value and gives
    /// black, whereas 100 is the value of diffuse white: it is perhaps possible to have a higher
    /// value for reflective surfaces.
    pub l: f64,
    /// The first opponent color axis. By convention, this is usually between -128 and 127, with -128
    /// being fully green and 127 being fully magenta, but note that it is still possible to create
    /// "imaginary" colors (ones that cannot normally be seen by the human eye). Additionally,
    /// depending on the other two dimensions, many colors with a value in this range will still not
    /// be in the range of human vision.
    pub a: f64,
    /// The second opponent color axis. This is, like `a`, between -128 and 127 by convention for most
    /// visible colors, although it is possible to work with imaginary colors as well and many colors
    /// with a value in this range are not in the range of human vision. -128 is fully blue; 127 is
    /// fully yellow.
    pub b: f64,
}

impl Color for CIELABColor {
    /// Converts a given CIE XYZ color to CIELAB. Because CIELAB is implicitly in a given illuminant
    /// space, and because the linear conversions within CIELAB that it uses conflict with the
    /// transform used in the rest of Scarlet, this is explicitly CIELAB D50: any other illuminant is
    /// converted to D50 outside of CIELAB conversion. This in line with programs like Photoshop,
    /// which also use CIELAB D50.
    fn from_xyz(xyz: XYZColor) -> CIELABColor {
        // TODO: are the bounds for a and b right? -128 to 127?
        // https://en.wikipedia.org/wiki/Lab_color_space#CIELAB-CIEXYZ_conversions
        let f = |x: &f64| {
            let delta: f64 = 6.0 / 29.0;
            if *x <= delta.powf(3.0) {
                x / (3.0 * delta * delta) + 4.0 / 29.0
            } else {
                x.powf(1.0 / 3.0)
            }
        };
        // now get the XYZ coordinates normalized using D50: convert to that beforehand if not
        let white_point = Illuminant::D50.white_point();
        let xyz_adapted = xyz.color_adapt(Illuminant::D50);
        let xyz_scaled = [
            xyz_adapted.x / white_point[0],
            xyz_adapted.y / white_point[1],
            xyz_adapted.z / white_point[2],
        ];
        let xyz_transformed: Vec<f64> = xyz_scaled.iter().map(f).collect();

        // xyz_transformed was modified to allow for human nonlinearity of color vision
        // so this is just simple linear formulae
        // note how a and b are opponent color axes
        let l = 116.0 * xyz_transformed[1] - 16.0;
        let a = 500.0 * (xyz_transformed[0] - xyz_transformed[1]);
        let b = 200.0 * (xyz_transformed[1] - xyz_transformed[2]);
        CIELABColor { l, a, b }
    }
    /// Returns an XYZ color that corresponds to the CIELAB color. Note that, because implicitly every
    /// CIELAB color is D50, conversion is done by first converting to a D50 XYZ color and then using
    /// a chromatic adaptation transform.
    fn to_xyz(&self, illuminant: Illuminant) -> XYZColor {
        // for implementation details see from_xyz
        // we need the inverse function of the nonlinearity we introduced earlier
        let f_inv = |x: f64| {
            let delta: f64 = 6.0 / 29.0;
            if x > delta {
                x * x * x
            } else {
                3.0 * delta * delta * (x - 4.0 / 29.0)
            }
        };
        // need to undo normalization with D50 white point
        let xyz_n = Illuminant::D50.white_point();
        let x = xyz_n[0] * f_inv((self.l + 16.0) / 116.0 + (self.a / 500.0));
        let y = xyz_n[1] * f_inv((self.l + 16.0) / 116.0);
        let z = xyz_n[2] * f_inv((self.l + 16.0) / 116.0 - (self.b / 200.0));
        // this is CIELAB D50, so to use custom illuminant do chromatic adaptation
        XYZColor {
            x,
            y,
            z,
            illuminant: Illuminant::D50,
        }
        .color_adapt(illuminant)
    }
}

impl From<Coord> for CIELABColor {
    fn from(c: Coord) -> CIELABColor {
        CIELABColor {
            l: c.x,
            a: c.y,
            b: c.z,
        }
    }
}

impl Into<Coord> for CIELABColor {
    fn into(self) -> Coord {
        Coord {
            x: self.l,
            y: self.a,
            z: self.b,
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use color::RGBColor;
    use consts::TEST_PRECISION;

    #[test]
    fn test_cielab_xyz_conversion_d50() {
        let xyz = XYZColor {
            x: 0.4,
            y: 0.2,
            z: 0.6,
            illuminant: Illuminant::D50,
        };
        let lab = CIELABColor::from_xyz(xyz);
        let xyz2 = lab.to_xyz(Illuminant::D50);
        assert!(xyz.approx_equal(&xyz2));
        assert!(xyz.distance(&xyz2) <= TEST_PRECISION);
    }
    #[test]
    fn test_cielab_xyz_conversion() {
        let xyz = XYZColor {
            x: 0.4,
            y: 0.2,
            z: 0.6,
            illuminant: Illuminant::D65,
        };
        let lab = CIELABColor::from_xyz(xyz);
        let xyz_d50 = lab.to_xyz(Illuminant::D50);
        let xyz2 = xyz_d50.color_adapt(Illuminant::D65);
        assert!(xyz.approx_equal(&xyz2));
        assert!(xyz.distance(&xyz2) <= TEST_PRECISION);
    }
    #[test]
    fn test_out_of_gamut() {
        // this color doesn't exist in sRGB! (that's probably a good thing, this can't really be represented)
        let _color1 = CIELABColor {
            l: 0.0,
            a: 100.0,
            b: 100.0,
        };
        let _color2: RGBColor = _color1.convert();
        let _color3: CIELABColor = _color2.convert();
    }
}
