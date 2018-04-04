//! This file implements the CIELCH color space, a cylindrical transformation of CIELAB that uses
//! chroma and hue instead of two opponent color axes. Be careful not to confuse this color with
//! CIEHCL, which uses CIELUV internally.

use color::{Color, XYZColor};
use coord::Coord;
use illuminants::Illuminant;
use super::cielabcolor::CIELABColor;

/// A cylindrical form of CIELAB, analogous to the relationship between HSL and RGB.
/// # Example
///
/// ```
/// # use scarlet::prelude::*;
/// # use scarlet::colors::CIELCHColor;
/// // hue-shift red to yellow, keeping same brightness: really ends up to be brown
/// let red = RGBColor{r: 0.7, g: 0.1, b: 0.1};
/// let red_lch: CIELCHColor = red.convert();
/// let mut yellow = red_lch;
/// yellow.h = yellow.h + 40.;
/// println!("{}", red.to_string());
/// println!("{}", yellow.convert::<RGBColor>().to_string());
/// // prints #B31A1A
/// //        #835000
/// ```
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CIELCHColor {
    /// The luminance component, identical to CIELAB's and CIELUV's. Ranges between 0 and 100.
    pub l: f64,
    /// The chroma component. Chroma is defined as the difference from the grayscale color of the same
    /// luminance (in CIELAB, essentially the distance away from the line a = b = 0). It is
    /// perceptually uniform in the sense that a gradient of chroma looks visually
    /// even. Importantly, it is not linear with respect to additive color mixing: superimposing two
    /// colors that are not of the exact same hue will not add together their chromas. In the
    /// cylindrical space, this is equivalent to radius. It ranges from 0 to roughly 150 for most
    /// colors that are physically possible, although keep in mind that the space is not a cylinder
    /// and for most luminance values chroma ranges much smaller.
    pub c: f64,
    /// The hue component, in degrees. The least complicated and the most familiar: essentially the
    /// angle in cylindrical coordinates, it ranges from 0 degrees to 360. 90 degrees corresponds to
    /// yellow, 180 corresponds to green, 270 to blue, and 360 to red.
    pub h: f64,
}

impl Color for CIELCHColor {
    /// Converts from XYZ to LCH by way of CIELAB.
    fn from_xyz(xyz: XYZColor) -> CIELCHColor {
        // first get LAB coordinates
        let lab = CIELABColor::from_xyz(xyz);
        let l = lab.l; // the same in both spaces
        // now we have to do some math
        // radius is sqrt(a^2 + b^2)
        // angle is atan2(a, b)
        // Rust does this ez
        let c = lab.b.hypot(lab.a);
        // don't forget to convert to degrees
        let unbounded_h = lab.b.atan2(lab.a).to_degrees();
        // and now add or subtract 360 to get within range (0, 360)
        // should only need to be done once
        let h = if unbounded_h < 0.0 {
            unbounded_h + 360.0
        } else if unbounded_h > 360.0 {
            unbounded_h - 360.0
        } else {
            unbounded_h
        };

        CIELCHColor { l, c, h }
    }
    /// Converts from LCH back to XYZ by way of CIELAB, chromatically adapting it as CIELAB does.
    fn to_xyz(&self, illuminant: Illuminant) -> XYZColor {
        // go back to a and b
        // more math: a = c cos h, b = c sin h
        // Rust also has something for this which is hella cool
        let (sin, cos) = self.h.to_radians().sin_cos();
        CIELABColor {
            l: self.l,
            a: self.c * cos,
            b: self.c * sin,
        }.to_xyz(illuminant)
    }
}

impl From<Coord> for CIELCHColor {
    fn from(c: Coord) -> CIELCHColor {
        CIELCHColor {
            l: c.x,
            c: c.y,
            h: c.z,
        }
    }
}

impl Into<Coord> for CIELCHColor {
    fn into(self) -> Coord {
        Coord {
            x: self.l,
            y: self.c,
            z: self.h,
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_lch_xyz_conversion_same_illuminant() {
        let xyz = XYZColor {
            x: 0.2,
            y: 0.42,
            z: 0.23,
            illuminant: Illuminant::D50,
        };
        let lch: CIELCHColor = xyz.convert();
        let xyz2: XYZColor = lch.convert();
        assert!(xyz2.approx_equal(&xyz));
    }
    #[test]
    fn test_lch_xyz_conversion_different_illuminant() {
        let xyz = XYZColor {
            x: 0.2,
            y: 0.42,
            z: 0.23,
            illuminant: Illuminant::D55,
        };
        let lch: CIELCHColor = xyz.convert();
        let xyz2: XYZColor = lch.convert();
        assert!(xyz2.approx_visually_equal(&xyz));
    }
}
