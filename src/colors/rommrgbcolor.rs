//! This module implements the Romm, or ProPhoto, RGB space. Unlike other RGB gamuts, ProPhoto trades
//! having some imaginary colors in its gamut (13% of it can't be seen) in exchange for a much wider
//! gamut than other RGB spaces (90% of CIELAB surface colors).
//!
//! The specification of this color space is... challenging. The forward side is fine, but there's no
//! inverse given. Scarlet mathematically calculates the inverse by using the tabulated primary XYZ values
//! as a basis for a change-of-basis matrix, scaling by the values of D50 reference white so (1, 1,
//! 1) maps to it. It also have to undo the nonlinearity and flare correction, which could still
//! contain small errors.

use bound::Bound;
use color::{Color, XYZColor};
use consts::ROMM_RGB_TRANSFORM as ROMM;
use consts::ROMM_RGB_TRANSFORM_LU as ROMM_LU;
use coord::Coord;
use illuminants::Illuminant;

/// A color in the ROMM RGB color space, also known as the ProPhoto RGB space. This is a very wide RGB
/// gamut, wider than both Adobe RGB and sRGB, but the tradeoff is that the colors it uses as
/// primaries aren't ones that actually exist on reflective objects in the real world.
/// # Example
/// How big is sRGB's gamut compared to ROMM RGB?
///
/// ```
/// # use scarlet::prelude::*;
/// # use scarlet::colors::ROMMRGBColor;
/// // Get the range (min, max) for each ROMM RGB component, using bright red, green, and
/// // blue. Technically, the primaries are different hues, but this is a rough estimate and good enough
/// // for this example. This probably throws it off a fair amount, but the ballpark is right. The
/// // other thing to keep in mind is that not all of ROMM RGB's gamut is actually visible, so this
/// // number makes ROMM RGB seem better than it is.
/// let black: ROMMRGBColor = RGBColor{r: 0., g: 0., b: 0.}.convert();
/// let red: ROMMRGBColor = RGBColor{r: 1., g: 0., b: 0.}.convert();
/// let green: ROMMRGBColor = RGBColor{r: 0., g: 1., b: 1.}.convert();
/// let blue: ROMMRGBColor = RGBColor{r: 0., g: 0., b: 1.}.convert();
/// let r_range = red.r - black.r;
/// let g_range = green.g - black.g;
/// let b_range = blue.b - black.b;
/// let percent_coverage = r_range * g_range * b_range * 100.;
/// assert!((percent_coverage - 15.57).abs() <= 0.01);
/// ```
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ROMMRGBColor {
    /// The red primary component, as a floating point. Ranges from 0 to 1 for most representable
    /// colors.
    pub r: f64,
    /// The green primary component, as a floating point. Ranges from 0 to 1 for most representable
    /// colors.
    pub g: f64,
    /// The blue primary component, as a floating point. Ranges from 0 to 1 for most representable
    /// colors.
    pub b: f64,
}

impl Color for ROMMRGBColor {
    /// Converts a given XYZ color to the closest representable ROMM RGB color. As the ROMM RGB space
    /// uses D50 as a reference white, any other illuminant is chromatically adapted first.
    fn from_xyz(xyz: XYZColor) -> ROMMRGBColor {
        // convert to D50
        let xyz_c = xyz.color_adapt(Illuminant::D50);

        // matrix multiplication, using spec's variable names
        // &* needed because lazy_static uses a different type which implements Deref
        let rr_gg_bb = *ROMM * vector![xyz_c.x, xyz_c.y, xyz_c.z];

        // like sRGB, there's a linear part and an exponential part to the gamma conversion
        let gamma = |x: f64| {
            // technically the spec I cite has a truncated version of the cutoff, but why not use the
            // exact one if it's a nicer format and probably causes fewer float issues
            if x < (2.0f64).powf(-9.0) {
                x * 16.0
            } else {
                x.powf(1.0 / 1.8)
            }
        };

        // as the spec describes, some "flare" can occur: to fix this, we apply a small fix so that
        // black is just really small and not 0
        let fix_flare = |x: f64| {
            if x < 0.03125 {
                0.003473 + 0.0622829 * x
            } else {
                0.003473 + 0.996527 * x.powf(1.8)
            }
        };

        // we also need to clamp between 0 and 1
        let clamp = |x: f64| {
            if x < 0.0 {
                0.0
            } else if x > 1.0 {
                1.0
            } else {
                x
            }
        };
        // now just apply these in sequence
        ROMMRGBColor {
            r: fix_flare(gamma(clamp(rr_gg_bb[0]))),
            g: fix_flare(gamma(clamp(rr_gg_bb[1]))),
            b: fix_flare(gamma(clamp(rr_gg_bb[2]))),
        }
    }
    /// Converts back from ROMM RGB to XYZ. As ROMM RGB uses D50, any other illuminant given will be
    /// chromatically adapted to from D50.
    /// This implementation is not from a spec: it's just the mathematical inverse of the from_xyz
    /// function, as best as the library author can compute it. This is the most likely function to
    /// give mismatches with other libraries or contain errors.
    fn to_xyz(&self, illuminant: Illuminant) -> XYZColor {
        // undo the gamma function, find the piecewise split
        let gamma_inv = |x: f64| {
            if x >= 0.03125 {
                // junction of two piecewise parts
                // this is the exponential part
                x.powf(1.8)
            } else {
                // this is the linear part
                x / 16.0
            }
        };

        // we have to first undo the fix_flare function: there's a different cutoff for the piecewise
        // function, because inputting 0.03125 doesn't produce 0.03125
        // WolframAlpha is my source for all of the calcluations
        let fix_flare_inv = |x: f64| {
            // fix_flare(2 ^ -9) is cutoff
            if x >= 0.005419340625 {
                // x originally came out of the second part of the cutoff
                ((x - 0.003473) / 0.996527).powf(1.0 / 1.8)
            } else {
                // x originally came out of the first part of the cutoff
                (x - 0.003473) / 0.0622829
            }
        };

        // now we undo gamma the same way

        let r_c = gamma_inv(fix_flare_inv(self.r));
        let g_c = gamma_inv(fix_flare_inv(self.g));
        let b_c = gamma_inv(fix_flare_inv(self.b));
        // The standard brilliantly decided to not even bother adding an inverse matrix. Scarlet uses
        // LU decomposition to avoid any precision loss when solving the equation for the right
        // values. This might differ from other solutions elsewhere: trust this one, unless you have
        // a good reason not to.
        let xyz = ROMM_LU
            .solve(&vector![r_c, g_c, b_c])
            .expect("Matrix is invertible.");
        // now we convert from D50 to whatever space we need and we're done!
        XYZColor {
            x: xyz[0],
            y: xyz[1],
            z: xyz[2],
            illuminant: Illuminant::D50,
        }
        .color_adapt(illuminant)
    }
}

impl From<Coord> for ROMMRGBColor {
    fn from(c: Coord) -> ROMMRGBColor {
        ROMMRGBColor {
            r: c.x,
            g: c.y,
            b: c.z,
        }
    }
}

impl From<ROMMRGBColor> for Coord {
    fn from(val: ROMMRGBColor) -> Self {
        Coord {
            x: val.r,
            y: val.g,
            z: val.b,
        }
    }
}

impl Bound for ROMMRGBColor {
    fn bounds() -> [(f64, f64); 3] {
        [(0., 1.), (0., 1.), (0., 1.)]
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use consts::TEST_PRECISION;

    #[test]
    fn test_romm_rgb_xyz_conversion() {
        let xyz = XYZColor {
            x: 0.4,
            y: 0.5,
            z: 0.6,
            illuminant: Illuminant::D50,
        };
        let rgb = ROMMRGBColor::from_xyz(xyz);
        let xyz2: XYZColor = rgb.to_xyz(Illuminant::D50);
        assert!(xyz.approx_equal(&xyz2));
        assert!(xyz.distance(&xyz2) <= TEST_PRECISION);
    }
    #[test]
    fn test_xyz_romm_rgb_conversion() {
        let rgb = ROMMRGBColor {
            r: 0.6,
            g: 0.3,
            b: 0.8,
        };
        let xyz = rgb.to_xyz(Illuminant::D50);
        let rgb2 = ROMMRGBColor::from_xyz(xyz);
        assert!((rgb.r - rgb2.r).abs() <= 0.001);
        assert!((rgb.g - rgb2.g).abs() <= 0.001);
        assert!((rgb.b - rgb2.b).abs() <= 0.001);
        assert!(rgb.distance(&rgb2) <= TEST_PRECISION);
    }
    #[test]
    fn test_error_accumulation_romm_rgb() {
        let rgb = ROMMRGBColor {
            r: 0.6,
            g: 0.3,
            b: 0.8,
        };
        let xyz = rgb.to_xyz(Illuminant::D50);
        let mut rgb2 = ROMMRGBColor::from_xyz(xyz);
        for _i in 0..20 {
            rgb2 = ROMMRGBColor::from_xyz(rgb2.to_xyz(Illuminant::D50));
            assert!((rgb.r - rgb2.r).abs() <= 1e-4);
            assert!((rgb.g - rgb2.g).abs() <= 1e-4);
            assert!((rgb.b - rgb2.b).abs() <= 1e-4);
            assert!(rgb.distance(&rgb2) <= TEST_PRECISION);
        }
    }

    #[test]
    fn test_romm_rgb_xyz_conversion_with_gamut() {
        let wp = Illuminant::D65.white_point();
        let xyz = XYZColor {
            x: wp[0],
            y: wp[1],
            z: wp[2],
            illuminant: Illuminant::D65,
        };
        let rgb: ROMMRGBColor = ROMMRGBColor::from_xyz(xyz);
        let xyz2 = rgb.to_xyz(Illuminant::D65);
        assert!(xyz.approx_visually_equal(&xyz2));
        assert!(xyz.distance(&xyz2) <= TEST_PRECISION);
    }
}
