//! A module that implements the Adobe RGB color space. The Adobe RGB space differs greatly from
//! sRGB: its components are floating points that range between 0 and 1, and it has a set of
//! primaries designed to give it a wider coverage (over half of CIE 1931).

use bound::Bound;
use coord::Coord;
use color::{Color, XYZColor};
use consts::ADOBE_RGB_TRANSFORM_MAT as ADOBE_RGB;
use consts;
use na::Vector3;
use illuminants::Illuminant;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
/// A color in the Adobe RGB color space. This is a rarer color space, but one that is still pretty
/// common, especially in color-managed design work. It can represent more colors than sRGB, which is
/// a plus if you have a monitor that can support it.
/// # Example
///
/// We can find the percentage of Adobe RGB that is inside the sRGB gamut:
/// Adobe RGB, in 3D space, is a cube 1 by 1 by 1. sRGB is within that a rectangular prism, so all we
/// need to do is just multiply together all of the sRGB ranges.
///
/// ```
/// # use scarlet::prelude::*;
/// # use scarlet::colors::AdobeRGBColor;
/// // Get the range (min, max) for each Adobe RGB component, using bright red, green, and
/// // blue. Technically, the primaries are different hues, but this is a rough estimate and good enough
/// // for this example.
/// let black: AdobeRGBColor = RGBColor{r: 0., g: 0., b: 0.}.convert();
/// let red: AdobeRGBColor = RGBColor{r: 1., g: 0., b: 0.}.convert();
/// let green: AdobeRGBColor = RGBColor{r: 0., g: 1., b: 1.}.convert();
/// let blue: AdobeRGBColor = RGBColor{r: 0., g: 0., b: 1.}.convert();
/// let r_range = red.r - black.r;
/// let g_range = green.g - black.g;
/// let b_range = blue.b - black.b;
/// let percent_coverage = r_range * g_range * b_range * 100.;
/// assert!((percent_coverage - 84.23).abs() <= 0.01);
/// ```
pub struct AdobeRGBColor {
    /// The red primary component. This is a float that should range between 0 and 1.
    pub r: f64,
    /// The green primary component. This is a float that should range between 0 and 1.
    pub g: f64,
    /// The blue primary component. This is a float that should range between 0 and 1.
    pub b: f64,
}

impl Color for AdobeRGBColor {
    /// Converts a given XYZ color to Adobe RGB. Adobe RGB is implicitly D65, so any color will be
    /// converted to D65 before conversion. Values outside of the Adobe RGB gamut will be clipped.
    fn from_xyz(xyz: XYZColor) -> AdobeRGBColor {
        // convert to D65
        let xyz_c = xyz.color_adapt(Illuminant::D65);
        // matrix multiplication
        // https://en.wikipedia.org/wiki/Adobe_RGB_color_space
        let rgb = ADOBE_RGB() * Vector3::new(xyz_c.x, xyz_c.y, xyz_c.z);

        // clamp
        let clamp = |x: f64| {
            if x > 1.0 {
                1.0
            } else if x < 0.0 {
                0.0
            } else {
                x
            }
        };

        // now we apply gamma transformation
        let gamma = |x: f64| x.powf(256.0 / 563.0);

        AdobeRGBColor {
            r: gamma(clamp(rgb[0])),
            g: gamma(clamp(rgb[1])),
            b: gamma(clamp(rgb[2])),
        }
    }
    /// Converts from Adobe RGB to an XYZ color in a given illuminant (via chromatic adaptation).
    fn to_xyz(&self, illuminant: Illuminant) -> XYZColor {
        // undo gamma transformation
        let ungamma = |x: f64| x.powf(563.0 / 256.0);

        // inverse matrix to the one in from_xyz
        let xyz_vec = consts::inv(ADOBE_RGB())
            * Vector3::new(ungamma(self.r), ungamma(self.g), ungamma(self.b));

        XYZColor {
            x: xyz_vec[0],
            y: xyz_vec[1],
            z: xyz_vec[2],
            illuminant: Illuminant::D65,
        }.color_adapt(illuminant)
    }
}

impl From<Coord> for AdobeRGBColor {
    fn from(c: Coord) -> AdobeRGBColor {
        AdobeRGBColor {
            r: c.x,
            g: c.y,
            b: c.z,
        }
    }
}

impl Into<Coord> for AdobeRGBColor {
    fn into(self) -> Coord {
        Coord {
            x: self.r,
            y: self.g,
            z: self.b,
        }
    }
}

impl Bound for AdobeRGBColor {
    fn bounds() -> [(f64, f64); 3] {
        [(0., 1.), (0., 1.), (0., 1.)]
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_adobe_rgb_xyz_conversion() {
        let xyz1 = XYZColor {
            x: 0.4,
            y: 0.2,
            z: 0.5,
            illuminant: Illuminant::D75,
        };
        let xyz2 = AdobeRGBColor::from_xyz(xyz1).to_xyz(Illuminant::D75);
        assert!(xyz1.approx_equal(&xyz2));
    }
    #[test]
    fn test_adobe_rgb_clamping() {
        let argb = AdobeRGBColor {
            r: 1.1,
            g: 0.6,
            b: 0.8,
        };
        let argb2 = AdobeRGBColor {
            r: 1.0,
            g: 0.6,
            b: 0.8,
        };
        let argbprime = argb.convert::<XYZColor>().convert::<AdobeRGBColor>();
        let argb2prime = argb2.convert::<XYZColor>().convert::<AdobeRGBColor>();
        let xyz1 = argbprime.to_xyz(Illuminant::D50);
        let xyz2 = argb2prime.to_xyz(Illuminant::D50);
        println!(
            "{} {} {} {} {} {}",
            xyz1.x, xyz2.x, xyz1.y, xyz2.y, xyz1.z, xyz2.z
        );
        assert!(xyz1.approx_equal(&xyz2));
    }
}
