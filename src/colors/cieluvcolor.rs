//! This module implements the CIELUV color specification, which was adopted concurrently with
//! CIELAB. CIELUV is very similar to CIELAB, but with the difference that u and v are roughly
//! equivalent to red and green and luminance is then used to calculate the blue part.

use color::{Color, XYZColor};
use coord::Coord;
use illuminants::Illuminant;

/// A similar color system to CIELAB, adapted at the same time and with similar goals. It attempts to
/// be an easy-to-convert color space from XYZ that approaches perceptual uniformity. U and V
/// represent chromaticity and roughly equate to CIELAB's A and B, but they're scaled differently and
/// act slightly differently. These coordinates are often referred to as the CIE 1976 UCS (uniform
/// chromaticity scale) diagram, and they're good descriptors of chromaticity.
/// # Example
///
/// ```
/// # use scarlet::prelude::*;
/// # use scarlet::colors::{CIELUVColor};
/// # use scarlet::color::XYZColor;
/// // D50 is the implied illuminant and white point
/// let white: CIELUVColor = XYZColor::white_point(Illuminant::D50).convert();
/// assert_eq!(white.l, 100.);
/// assert_eq!(white.u, 0.);
/// assert_eq!(white.v, 0.);
/// ```
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CIELUVColor {
    /// The luminance component of LUV. Ranges from 0 to 100 by definition.
    pub l: f64,
    /// The component of LUV that roughly equates to how red the color is vs. how green it is. Ranges
    /// from 0 to 100 in most visible colors, where 0 is bright green and 100 is bright red.
    pub u: f64,
    /// The component of LUV that roughly equates to how yellow vs. blue the color is. Ranges from 0 to
    /// 100 in most visible colors, where 0 is bright blue and 100 is bright yellow.
    pub v: f64,
}

impl Color for CIELUVColor {
    /// Given an XYZ color, gets a new CIELUV color. This is CIELUV D50, so anything else is
    /// chromatically adapted before conversion.
    fn from_xyz(xyz: XYZColor) -> CIELUVColor {
        // this is not bad: LUV is meant to be easy from XYZ
        // https://en.wikipedia.org/wiki/CIELUV

        // do u and v chromaticity conversions on whitepoint and on given color
        // because cieluv chromatic adaptation sucks, use the good one
        let xyz_c = xyz.color_adapt(Illuminant::D50);
        let wp = XYZColor::white_point(Illuminant::D50);
        let denom = |color: XYZColor| color.x + 15.0 * color.y + 3.0 * color.z;
        let u_func = |color: XYZColor| 4.0 * color.x / denom(color);
        let v_func = |color: XYZColor| 9.0 * color.y / denom(color);

        let u_prime_n = u_func(wp);
        let v_prime_n = v_func(wp);

        let u_prime = u_func(xyz_c);
        let v_prime = v_func(xyz_c);

        let delta: f64 = 6.0 / 29.0; // like CIELAB

        // technically this next division should do nothing: idk if it gets factored out at compile
        // time, but it's just insurance if someone ever decides not to normalize whitepoints to Y=1
        let y_scaled = xyz_c.y / wp.y; // ranges from 0-1
        let l = if y_scaled <= delta.powf(3.0) {
            (2.0 / delta).powf(3.0) * y_scaled
        } else {
            116.0 * y_scaled.powf(1.0 / 3.0) - 16.0
        };

        let u = 13.0 * l * (u_prime - u_prime_n);
        let v = 13.0 * l * (v_prime - v_prime_n);
        CIELUVColor { l, u, v }
    }
    /// Returns a new `XYZColor` that matches the given color. Note that Scarlet uses CIELUV D50 to
    /// get around compatibility issues, so any other illuminant will be chromatically adapted after
    /// initial conversion (using the `color_adapt()` function).
    fn to_xyz(&self, illuminant: Illuminant) -> XYZColor {
        // https://en.wikipedia.org/wiki/CIELUV literally has the equations in order
        // pretty straightforward
        let wp = XYZColor::white_point(Illuminant::D50);
        let denom = |color: XYZColor| color.x + 15.0 * color.y + 3.0 * color.z;
        let u_func = |color: XYZColor| 4.0 * color.x / denom(color);
        let v_func = |color: XYZColor| 9.0 * color.y / denom(color);
        let u_prime_n = u_func(wp);
        let v_prime_n = v_func(wp);

        let u_prime = self.u / (13.0 * self.l) + u_prime_n;
        let v_prime = self.v / (13.0 * self.l) + v_prime_n;

        let delta: f64 = 6.0 / 29.0;

        let y = if self.l <= 8.0 {
            wp.y * self.l * (delta / 2.0).powf(3.0)
        } else {
            wp.y * ((self.l + 16.0) / 116.0).powf(3.0)
        };

        let x = y * 9.0 * u_prime / (4.0 * v_prime);
        let z = y * (12.0 - 3.0 * u_prime - 20.0 * v_prime) / (4.0 * v_prime);
        XYZColor {
            x,
            y,
            z,
            illuminant: Illuminant::D50,
        }.color_adapt(illuminant)
    }
}

impl From<Coord> for CIELUVColor {
    fn from(c: Coord) -> CIELUVColor {
        CIELUVColor {
            l: c.x,
            u: c.y,
            v: c.z,
        }
    }
}

impl Into<Coord> for CIELUVColor {
    fn into(self) -> Coord {
        Coord {
            x: self.l,
            y: self.u,
            z: self.v,
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use mix::Mix;

    #[test]
    fn test_cieluv_xyz_conversion_d50() {
        let xyz = XYZColor {
            x: 0.3,
            y: 0.53,
            z: 0.65,
            illuminant: Illuminant::D50,
        };
        let luv: CIELUVColor = xyz.convert();
        let xyz2: XYZColor = luv.convert();
        assert!(xyz2.approx_equal(&xyz));
    }

    #[test]
    fn test_cieluv_xyz_conversion_d65() {
        let xyz = XYZColor {
            x: 0.3,
            y: 0.53,
            z: 0.65,
            illuminant: Illuminant::D65,
        };
        let luv: CIELUVColor = xyz.convert();
        let xyz2: XYZColor = luv.convert();
        assert!(xyz2.approx_visually_equal(&xyz));
    }

    fn test_cieluv_color_mixing() {
        let luv = CIELUVColor {
            l: 45.0,
            u: 67.0,
            v: 49.0,
        };
        let luv2 = CIELUVColor {
            l: 53.0,
            u: 59.0,
            v: 3.0,
        };
        let luv_mixed = luv.mix(luv2);
        assert!((luv_mixed.l - 49.0).abs() <= 1e-7);
        assert!((luv_mixed.u - 63.0).abs() <= 1e-7);
        assert!((luv_mixed.v - 26.0).abs() <= 1e-7);
    }
}
