//! This module implements the HSV color space, a cousin of the HSL color space. The definition of
//! value differs from lightness: it goes from black to full saturation instead of black to
//! white. This makes value an extraordinarily poor analog of luminance (dark purple is the same
//! value as white, despite reflecting one-tenth the light), but does make the hue and saturation a
//! bit more meaningful than HSL. The same caveat applies: this is a poor choice for getting actual
//! color appearance parameters and is outclassed by CIELCH for that purpose, but it is nontheless
//! important as the closest to such a space one can get using only basic transformations of RGB.

use bound::Bound;
use coord::Coord;
use color::{Color, RGBColor, XYZColor};
use illuminants::Illuminant;

/// An HSV color, defining parameters for hue, saturation, and value from the RGB space. This is sHSV
/// to be exact, but the derivation from the sRGB space is assumed as it matches the vast majority of
/// colors called RGB.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct HSVColor {
    /// The hue, described as an angle that ranges between 0 and 360 in degrees. While values outside
    /// of this range *may* not break, they shouldn't be treated as valid.
    pub h: f64,
    /// The saturation, defined as the radius of the HSV cylinder and the distance between the color
    /// and the equivalent-value grayscale. Ranges between 0 and 1.
    pub s: f64,
    /// The value, defined as the largest RGB primary value of a color. This corresponds to something
    /// close to color intensity, not really luminance: dark purple and white are the same value, for
    /// example.
    pub v: f64,
}

impl Color for HSVColor {
    /// Converts to HSV by going through sRGB.
    fn from_xyz(xyz: XYZColor) -> HSVColor {
        let rgb = RGBColor::from_xyz(xyz);

        // I call this chroma, but it's a very very rough estimate of the actual color attribute.
        // More info: https://en.wikipedia.org/wiki/HSL_and_HSV#Formal_derivation
        let components = [rgb.r, rgb.g, rgb.b];
        let max_c = components.iter().cloned().fold(-1.0, f64::max);
        let min_c = components.iter().cloned().fold(2.0, f64::min);
        let chroma = max_c - min_c;

        // hue is crazy in a hexagon! no more trig functions for us!
        // it's technically the proportion of the length of the hexagon through the point, but it's
        // treated as degrees
        let hue = if chroma == 0.0 {
            // could be anything, undefined according to Wikipedia, in Scarlet just 0 for gray
            0.0
        } else if max_c == rgb.r {
            // in red sector: find which part by comparing green and blue and scaling
            // adding green moves up on the hexagon, adding blue moves down: hence, linearity
            // the modulo makes sure it's in the range 0-360
            (((rgb.g - rgb.b) / chroma) % 6.0) * 60.0
        } else if max_c == rgb.g {
            // similar to above, but you add an offset
            ((rgb.b - rgb.r) / chroma) * 60.0 + 120.0
        } else {
            // same as above, different offset
            ((rgb.r - rgb.g) / chroma) * 60.0 + 240.0
        };

        // saturation, scientifically speaking, is chroma adjusted for lightness. For HSL, it's
        // defined relative to the maximum chroma, which varies depending on the place on the
        // cone. Thus, I'll compute lightness first.

        // now we use value: the largest component
        let value = max_c;
        // now back to saturation
        let saturation = if value == 0.0 {
            // this would be a divide by 0 otherwise, just set it to 0 because it doesn't matter
            0.0
        } else {
            chroma / value
        };

        HSVColor {
            h: hue,
            s: saturation,
            v: value,
        }
    }
    /// Converts from HSV back to XYZ. Any illuminant other than D65 is computed using chromatic adaptation.
    fn to_xyz(&self, illuminant: Illuminant) -> XYZColor {
           // first get back chroma

        let chroma = self.s * self.v;
        // find the point with 0 lightness that matches ours in the other two components

        // intermediate value is the second-largest RGB value, where C is the largest because the
        // smallest is 0: call this x
        let x = chroma * (1.0 - ((self.h / 60.0) % 2.0 - 1.0).abs());
        // now split based on which line of the hexagon we're on, i.e., which are the two largest
        // components
        let (r1, g1, b1) = if self.h <= 60.0 {
            (chroma, x, 0.0)
        } else if self.h <= 120.0 {
            (x, chroma, 0.0)
        } else if self.h <= 180.0 {
            (0.0, chroma, x)
        } else if self.h <= 240.0 {
            (0.0, x, chroma)
        } else if self.h <= 300.0 {
            (x, 0.0, chroma)
        } else {
            (chroma, 0.0, x)
        };
        // now we add the right value to each component to get the correct lightness and scale back
        // to 0-255
        let offset = self.v - chroma;
        let r = r1 + offset;
        let g = g1 + offset;
        let b = b1 + offset;
        RGBColor { r, g, b }.to_xyz(illuminant)
    }
}

impl From<Coord> for HSVColor {
    fn from(c: Coord) -> HSVColor {
        HSVColor {
            h: c.x,
            s: c.y,
            v: c.z,
        }
    }
}

impl Into<Coord> for HSVColor {
    fn into(self) -> Coord {
        Coord {
            x: self.h,
            y: self.s,
            z: self.v,
        }
    }
}

impl Bound for HSVColor {
    fn bounds() -> [(f64, f64); 3] {
        [(0., 360.), (0., 1.), (0., 1.)]
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use color::Mix;

    #[test]
    fn test_hsl_rgb_conversion() {
        let red_rgb = RGBColor {
            r: 1.,
            g: 0.,
            b: 0.,
        };
        let red_hsv: HSVColor = red_rgb.convert();
        println!("{}", red_hsv.s);
        assert!(red_hsv.h.abs() <= 0.0001);
        assert!((red_hsv.s - 1.0) <= 0.0001);
        assert!((red_hsv.v - 1.0) <= 0.0001);
        let lavender_hsv = HSVColor {
            h: 243.5,
            s: 0.568,
            v: 0.925,
        };
        let lavender_rgb: RGBColor = lavender_hsv.convert();
        assert_eq!(lavender_rgb.to_string(), "#6E66EC");
    }
    #[test]
    fn test_hsv_color_mixing() {
        // red mixed with green should be yellow, as little sense as that makes
        let red = HSVColor {
            h: 0.0,
            s: 1.0,
            v: 1.0,
        };
        let green = HSVColor {
            h: 120.0,
            s: 1.0,
            v: 1.0,
        };
        let yellow = HSVColor {
            h: 60.0,
            s: 1.0,
            v: 1.0,
        };
        let mixed = red.mix(green);
        assert!((mixed.h - yellow.h).abs() <= 0.0001);
        assert!((mixed.s - yellow.s).abs() <= 0.0001);
        assert!((mixed.v - yellow.v).abs() <= 0.0001);

        // test with all three components changing
        let hsv1 = HSVColor {
            h: 234.0,
            s: 0.6,
            v: 0.7,
        };
        let hsv2 = HSVColor {
            h: 134.0,
            s: 1.0,
            v: 0.5,
        };
        let hsv3 = HSVColor {
            h: 184.0,
            s: 0.8,
            v: 0.6,
        };
        let hsv4 = hsv1.mix(hsv2);
        assert!((hsv4.h - hsv3.h).abs() <= 0.0001);
        assert!((hsv4.s - hsv3.s).abs() <= 0.0001);
        assert!((hsv4.v - hsv3.v).abs() <= 0.0001);
    }
}


