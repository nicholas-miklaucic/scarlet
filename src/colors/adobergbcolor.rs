//! A module that implements the Adobe RGB color space. The Adobe RGB space differs greatly from
//! sRGB: its components are floating points that range between 0 and 1, and it has a set of
//! primaries designed to give it a wider coverage (over half of CIE 1931).

use coord::Coord;
use color::{Color, XYZColor};
use illuminants::Illuminant;

#[derive(Debug, Copy, Clone)]
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
        // essentially matrix multiplication
        // https://en.wikipedia.org/wiki/Adobe_RGB_color_space
        let r = 2.04159 * xyz_c.x - 0.56501 * xyz_c.y - 0.34473 * xyz_c.z;
        let g = -0.96924 * xyz_c.x + 1.87597 * xyz_c.y + 0.04156 * xyz_c.z;
        let b = 0.01344 * xyz_c.x - 0.11836 * xyz_c.y + 1.01517 * xyz_c.z;

        // clamp
        let clamp = |x: f64| {
            if x > 1.0 {
                1.0
            }
            else if x < 0.0 {
                0.0
            }
            else {
                x
            }
        };
        
        // now we apply gamma transformation
        let gamma = |x: f64| {
            x.powf(256.0 / 563.0)
        };

        AdobeRGBColor{
            r: gamma(clamp(r)),
            g: gamma(clamp(g)),
            b: gamma(clamp(b)),
        }
    }
    /// Converts from Adobe RGB to an XYZ color in a given illuminant (via chromatic adaptation).
    fn to_xyz(&self, illuminant: Illuminant) -> XYZColor {
        // undo gamma transformation
        let ungamma = |x: f64| {
            x.powf(563.0 / 256.0)
        };

        // inverse matrix to the one in from_xyz
        let x = 0.57667 * ungamma(self.r) + 0.18556 * ungamma(self.g) + 0.18823 * ungamma(self.b);
        let y = 0.29734 * ungamma(self.r) + 0.62736 * ungamma(self.g) + 0.07529 * ungamma(self.b);
        let z = 0.02703 * ungamma(self.r) + 0.07069 * ungamma(self.g) + 0.99134 * ungamma(self.b);

        let xyz = XYZColor{x, y, z, illuminant: Illuminant::D65};
        xyz.color_adapt(illuminant)
    }
}

impl From<Coord> for AdobeRGBColor {
    /// Converts from a Coordinate (R, G, B) to a color. Clamps values outside of the range [0, 1].
    fn from(c: Coord) -> AdobeRGBColor {
        // clamp values
        let clamp = |x: f64| {
            if x <= 0.0 {
                0.0
            }
            else if x >= 1.0 {
                1.0
            }
            else {
                x
            }
        };
        AdobeRGBColor{r: clamp(c.x), g: clamp(c.y), b: clamp(c.z)}
    }
}

impl Into<Coord> for AdobeRGBColor {
    fn into(self) -> Coord {
        Coord{x: self.r, y: self.g, z: self.b}
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use color::Mix;

    #[test]
    fn test_adobe_rgb_xyz_conversion() {
        let xyz1 = XYZColor{x: 0.4, y: 0.2, z: 0.5, illuminant: Illuminant::D75};
        let xyz2 = AdobeRGBColor::from_xyz(xyz1).to_xyz(Illuminant::D75);
        assert!(xyz1.approx_equal(&xyz2));
    }
    #[test]
    fn test_adobe_rgb_clamping() {
        let argb = AdobeRGBColor{r: 1.1, g: 0.6, b: 0.8};
        let argb2 = AdobeRGBColor{r: 1.0, g: 0.6, b: 0.8};
        let argbprime = argb.convert::<XYZColor>().convert::<AdobeRGBColor>();
        let argb2prime = argb2.convert::<XYZColor>().convert::<AdobeRGBColor>();
        let xyz1 = argbprime.to_xyz(Illuminant::D50);
        let xyz2 = argb2prime.to_xyz(Illuminant::D50);
        println!("{} {} {} {} {} {}", xyz1.x, xyz2.x, xyz1.y, xyz2.y, xyz1.z, xyz2.z);
        assert!(xyz1.approx_equal(&xyz2));
    }
    #[test]
    fn test_adobe_rgb_mixing() {
        let argb = AdobeRGBColor{r: 0.4, g: 0.2, b: 0.5};
        let argb2 = AdobeRGBColor{r: 0.6, g: 0.6, b: 0.8};
        let argb_mixed = argb.mix(argb2);
        assert!((argb_mixed.r - 0.5).abs() <= 1e-7);
        assert!((argb_mixed.g - 0.4).abs() <= 1e-7);
        assert!((argb_mixed.b - 0.65).abs() <= 1e-7);
    }
}
