/// This file defines the Color trait and all of the standard color types that implement it.

use std::convert::From;
use std::num::ParseIntError;
use std::result::Result::Err;
use std::string::ToString;

use super::coord::Coord;
use illuminants::{Illuminant};

extern crate termion;
use self::termion::color::{Fg, Bg, Reset, Rgb};



/// A point in the CIE 1931 XYZ color space. Although any point in XYZ coordinate space is technically
/// valid, in this library XYZ colors are treated as normalized so that Y=1 is the white point of
/// whatever illuminant is being worked with.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct XYZColor {
    // these need to all be positive
    // TODO: way of implementing this constraint in code?
    /// The X axis of the CIE 1931 XYZ space, roughly representing the long-wavelength receptors in
    /// the human eye: the red receptors. Usually between 0 and 1, but can range more than that.
    pub x: f64,
    /// The Y axis of the CIE 1931 XYZ space, roughly representing the middle-wavelength receptors in
    /// the human eye. In CIE 1931, this is fudged to correspond exactly with perceived luminance.
    pub y: f64,
    /// The Z axis of the CIE 1931 XYZ space, roughly representing the short-wavelength receptors in
    /// the human eye. Usually between 0 and 1, but can range more than that.
    pub z: f64,
    /// The illuminant that is assumed to be the lighting environment for this color. Although XYZ
    /// itself describes the human response to a color and so is independent of lighting, it is useful
    /// to consider the question "how would an object in one light look different in another?" and so,
    /// to contain all the information needed to track this, the illuminant is set. Don't modify this
    /// directly in most cases: use the `color_adapt` function to do it.
    pub illuminant: Illuminant,
}

impl XYZColor {
    // Transforms a given XYZ coordinate to LMS space, returning an array [L, M, S] given an array
    // [X, Y, Z]. Information about this is given in the color_adapt function, which is the only
    // public function that uses this one right now. Uses the linearized Bradford transform.
    fn lms_transform(xyz: [f64; 3]) -> [f64; 3] {
        let l = 0.8951 * xyz[0] + 0.2664 * xyz[1] - 0.1614 * xyz[2];
        let m = -0.7502 * xyz[0] + 1.7135 * xyz[1] + 0.0367 * xyz[2];
        let s = 0.0389 * xyz[0] - 0.0685 * xyz[1] + 1.0296 * xyz[2];
        [l, m, s]
    }
    /// Performs *color adaptation*: attempts to convert the given color to one that would look the
    /// exact same under a different illumination. Chromatic adapation is a nontrivial task, and
    /// there is no one correct way to do this. Scarlet uses a *linearized Bradford von Kries
    /// transform*, which is technically incorrect but has several advantages over "better"
    /// transforms like the CIECAT02 transform. Specifically, it doesn't require any information
    /// about the surrounding absolute luminance, which is pretty much impossible to know anything
    /// about outside of color science and psychophysics. More information can be found
    /// [here.](https://en.wikipedia.org/wiki/Chromatic_adaptation)
    pub fn color_adapt(&self, other_illuminant: Illuminant) -> XYZColor {
        // no need to transform if same illuminant
        if other_illuminant == self.illuminant {
            *self
        }
        else {
            // convert to LMS color space using matrix multiplication
            // this is called spectral sharpening, intended to increase clarity
            let lms = XYZColor::lms_transform([self.x, self.y, self.z]);

            // get the LMS values for the white point of the illuminant we are currently using and
            // the one we want: wr here stands for "white reference", i.e., the one we're converting
            // to
            let lms_w = XYZColor::lms_transform(self.illuminant.white_point());
            let lms_wr = XYZColor::lms_transform(other_illuminant.white_point());

            // perform the transform
            // this usually includes a parameter indicating how much you want to adapt, but it's
            // assumed that we want total adaptation: D = 1. Maybe this could change someday?

            // because each white point has already been normalized to Y = 1, we don't need a
            // factor for it, which simplifies calculation even more than setting D = 1 and makes it
            // just a linear transform
            let l_c = lms_wr[0] * lms[0] / lms_w[0];
            let m_c = lms_wr[1] * lms[1] / lms_w[1];
            let s_c = lms_wr[2] * lms[2] / lms_w[2];

            // now we convert right back into XYZ space, using the inverse of the LMS matrix we used
            // earlier. Because we only do this once, this isn't its own function. As with the other
            // function, this uses the Bradford values found at
            // http://onlinelibrary.wiley.com/doi/10.1002/9781119021780.app3/pdf
            let x_c = 0.9870 * l_c - 0.1471 * m_c + 0.1600 * s_c;
            let y_c = 0.4323 * l_c + 0.5184 * m_c + 0.0493 * s_c;
            let z_c = -0.0085 * l_c + 0.0400 * m_c + 0.9685 * s_c;

            // the full CIECAT02 model now would do a post-adaptation transform, but we're just going
            // to ignore that because we don't have any absolute luminance information.
            XYZColor{x: x_c, y: y_c, z: z_c, illuminant: other_illuminant}
        }
    }
    /// Returns `true` if the given other XYZ color's coordinates are all within 0.001 of each other,
    /// which helps account for necessary floating-point errors in conversions.
    pub fn approx_equal(&self, other: &XYZColor) -> bool {
        ((self.x - other.x).abs() <= 0.001 &&
         (self.y - other.y).abs() <= 0.001 &&
         (self.z - other.z).abs() <= 0.001)
    }
        
    /// Returns `true` if the given other XYZ color would look identically in a different color
    /// space. Uses an approximate float equality that helps resolve errors due to floating-point
    /// representation, only testing if the two floats are within 0.001 of each other.
    pub fn approx_visually_equal(&self, other: &XYZColor) -> bool {
        let other_c = other.color_adapt(self.illuminant);
        self.approx_equal(&other_c)
    }
    /// Gets the XYZColor corresponding to pure white in the given light environment.
    pub fn white_point(illuminant: Illuminant) -> XYZColor {
        let wp = illuminant.white_point();
        XYZColor{x: wp[0], y: wp[1], z: wp[2], illuminant}
    }
}

/// A trait that includes any color representation that can be converted to and from the CIE 1931 XYZ
/// color space.
pub trait Color {
    /// Converts from a color in CIE 1931 XYZ to the given color type.
    fn from_xyz(XYZColor) -> Self;
    /// Converts from the given color type to a color in CIE 1931 XYZ space. Because most color types
    /// don't include illuminant information, it is provided instead, as an enum. For most
    /// applications, D50 or D65 is a good choice.
    fn to_xyz(&self, illuminant: Illuminant) -> XYZColor;

    /// Converts the given Color to a different Color type, without consuming the curreppnt color. `T`
    /// is the color that is being converted to.  This currently converts back and forth using the
    /// D50 standard illuminant. However, this shouldn't change the actual value if the color
    /// conversion methods operate correctly, and this value should not be relied upon and can be
    /// changed without notice.
    fn convert<T: Color>(&self) -> T {
        // theoretically, the illuminant shouldn't matter as long as the color conversions are
        // correct. D50 is a common gamut for use in internal conversions, so for spaces like CIELAB
        // it will produce the least error
        T::from_xyz(self.to_xyz(Illuminant::D50))
    }
    /// "Colors" a given piece of text with terminal escape codes to allow it to be printed out in the
    /// given foreground color. Will cause problems with terminals that do not support truecolor.
    fn write_colored_str(&self, text: &str) -> String {
        let rgb: RGBColor = self.convert();
        rgb.base_write_colored_str(text)
    }
    /// Returns a string which, when printed in a truecolor-supporting terminal, will hopefully have
    /// both the foreground and background of the desired color, appearing as a complete square.
    fn write_color(&self) -> String {
        let rgb: RGBColor = self.convert();
        rgb.base_write_color()
    }
}

impl Color for XYZColor {
    fn from_xyz(xyz: XYZColor) -> XYZColor {
        xyz
    }
    #[allow(unused_variables)]
    fn to_xyz(&self, illuminant: Illuminant) -> XYZColor {
        *self
    }
}

#[derive(Debug, Copy, Clone, Eq)]
pub struct RGBColor {
    r: u8,
    g: u8,
    b: u8,
    // TODO: add exact unclamped versions of each of these
}
    
impl RGBColor {
    /// Given a string, returns that string wrapped in codes that will color the foreground. Used for
    /// the trait implementation of write_colored_str, which should be used instead.
    fn base_write_colored_str(&self, text: &str) -> String {
        format!("{code}{text}{reset}",
                code=Fg(Rgb(self.r, self.g, self.b)),
                text=text,
                reset=Fg(Reset)
        )
    }
    fn base_write_color(&self) -> String {
        format!("{bg}{fg}{text}{reset_fg}{reset_bg}",
                bg=Bg(Rgb(self.r, self.g, self.b)),
                fg=Fg(Rgb(self.r, self.g, self.b)),
                text="■",
                reset_fg=Fg(Reset),
                reset_bg=Bg(Reset),
        )
    }
}
// TODO: get RGB from string

impl PartialEq for RGBColor {
    fn eq(&self, other: &RGBColor) -> bool {
        self.r == other.r && self.g == other.g && self.b == other.b
    }
}
        

impl From<(u8, u8, u8)> for RGBColor {
    fn from(rgb: (u8, u8, u8)) -> RGBColor {
        let (r, g, b) = rgb;
        RGBColor{r, g, b}
    }
}

impl Into<(u8, u8, u8)> for RGBColor {
    fn into(self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
}

impl ToString for RGBColor {
    fn to_string(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

impl Color for RGBColor {
    fn from_xyz(xyz: XYZColor) -> RGBColor {
        // sRGB uses D65 as the assumed illuminant: convert the given value to that
        let xyz_d65 = xyz.color_adapt(Illuminant::D65);
        // first, get linear RGB values (i.e., without gamma correction)
        // https://en.wikipedia.org/wiki/SRGB#Specification_of_the_transformation

        // note how the diagonals are large: X, Y, Z, roughly equivalent to R, G, B
        let rgb_lin_vec = vec![3.2406 * xyz_d65.x - 1.5372 * xyz_d65.y - 0.4986 * xyz_d65.z,
                               -0.9689 * xyz_d65.x + 1.8758 * xyz_d65.y + 0.0415 * xyz_d65.z,
                               0.0557 * xyz_d65.x - 0.2040 * xyz_d65.y + 1.0570 * xyz_d65.z];
        // now we scale for gamma correction
        let gamma_correct = |x: &f64| {
            if x <= &0.0031308 {
                &12.92 * x
            }
            else {
                &1.055 * x.powf(&1.0 / &2.4) - &0.055
            }
        };
        let float_vec:Vec<f64> = rgb_lin_vec.iter().map(gamma_correct).collect();
        // now rescale between 0 and 255 and cast to integers
        // TODO: deal with clamping and exact values
        // we're going to clamp values to between 0 and 255
        let clamp = |x: &f64| {
            if *x >= 1.0 {
                1.0
            } else if *x <= 0.0 {
                0.0
            } else {
                *x
            }
        };
        let rgb:Vec<u8> = float_vec.iter().map(clamp).map(|x| (x * 255.0).round() as u8).collect();
        
        RGBColor {
            r: rgb[0],
            g: rgb[1],
            b: rgb[2]
        }
    }

    fn to_xyz(&self, illuminant: Illuminant) -> XYZColor {
        // scale from 0 to 1 instead
        // TODO: use exact values here?
        let uncorrect_gamma = |x: &f64| {
            if x <= &0.04045 {
                x / &12.92
            }
            else {
                ((x + &0.055) / &1.055).powf(2.4)
            }
        };
        let scaled_vec: Vec<f64> = vec![self.r, self.g, self.b].iter().map(|x| (*x as f64) / 255.0).collect();
        let rgb_vec: Vec<f64> = scaled_vec.iter().map(uncorrect_gamma).collect();

        // essentially the inverse of the above matrix multiplication
        let x = 0.4124 * rgb_vec[0] + 0.3576 * rgb_vec[1] + 0.1805 * rgb_vec[2];
        let y = 0.2126 * rgb_vec[0] + 0.7152 * rgb_vec[1] + 0.0722 * rgb_vec[2];
        let z = 0.0193 * rgb_vec[0] + 0.1192 * rgb_vec[1] + 0.9505 * rgb_vec[2];

        // sRGB, which this is based on, uses D65 as white, but you can convert to whatever
        // illuminant is specified
        let converted = XYZColor{x, y, z, illuminant: Illuminant::D65};
        converted.color_adapt(illuminant)        
    }
}

/// An error type that results from an invalid attempt to convert a string into an RGB color.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum RGBParseError {
    /// This indicates that function syntax was acceptable, but the numbers were out of range, such as
    /// the invalid string `"rgb(554, 23, 553)"`.
    OutOfRange,
    /// This indicates that the hex string was malformed in some way.
    InvalidHexSyntax,
    /// This indicates a syntax error in the string that was supposed to be a valid rgb( function.
    InvalidFuncSyntax,
}

impl From<ParseIntError> for RGBParseError {
    fn from(_err: ParseIntError) -> RGBParseError {
        RGBParseError::OutOfRange
    }
}

impl RGBColor {
    /// Given a string that represents a hex code, returns the RGB color that the given hex code
    /// represents. Four formats are accepted: `"#rgb"` as a shorthand for `"#rrggbb"`, `#rrggbb` by
    /// itself, and either of those formats without `#`: `"rgb"` or `"rrggbb"` are acceptable. Returns
    /// a ColorParseError if the given string does not follow one of these formats.
    pub fn from_hex_code(hex: &str) -> Result<RGBColor, RGBParseError> {
        let mut chars: Vec<char> = hex.chars().collect();
        // check if leading hex, remove if so
        if chars[0] == '#' {
            chars.remove(0);
        }
        println!("{:?}", chars);
        // can only have 3 or 6 characters: error if not so
        if chars.len() != 3 && chars.len() != 6 {
            Err(RGBParseError::InvalidHexSyntax)
            // now split on invalid hex
        } else if !chars.iter().all(|&c| "0123456789ABCDEFabcdef".contains(c)) {
            Err(RGBParseError::InvalidHexSyntax)
        } else {
            // split on whether it's #rgb or #rrggbb
            if chars.len() == 6 {
                let mut rgb: Vec<u8> = Vec::new();
                for _i in 0..3 {
                    // this should never fail, logically, but if by some miracle it did it'd just
                    // return an OutOfRangeError
                    rgb.push(u8::from_str_radix(chars.drain(..2).collect::<String>().as_str(), 16).unwrap());
                }
                Ok(RGBColor{r: rgb[0], g: rgb[1], b: rgb[2]})
            }
            else { // len must be 3 from earlier
                let mut rgb: Vec<u8> = Vec::new();
                for _i in 0..3 {
                    // again, this shouldn't ever fail, but if it did it'd just return an
                    // OutOfRangeError
                    let c: Vec<char> = chars.drain(..1).collect();
                    rgb.push(u8::from_str_radix(c.iter().chain(c.iter()).collect::<String>().as_str(), 16).unwrap());
                }
                Ok(RGBColor{r: rgb[0], g: rgb[1], b: rgb[2]})
            }
        }
    }
}

/// Describes a Color that can be mixed with other colors in its own 3D space. Mixing, in this
/// context, is taking the midpoint of two color projections in some space, or something consistent
/// with that idea: if colors A and B mix to A, that should mean B is the same as A, for
/// example. Although this is not currently the case, note that this implies that the gamut of this
/// Color is convex: any two Colors of the same type may be mixed to form a third valid one.

/// Note that there is one very crucial thing to remember about mixing: it differs depending on the
/// color space being used. For example, if there are two colors A and B, A.mix(B) may produce very
/// different results than A_conv.mix(B_conv) if A_conv and B_conv are the results of A.convert() and
/// B.convert(). For this reason, A.mix(B) is only allowed if A and B share a type: otherwise,
/// A.mix(B) could be different than B.mix(A), which is error-prone and unintuitive.

/// There is a default implementation for Colors that can interconvert to Coord. This helps ensure
/// that the most basic case functions appropriately. For any other type of Color, special logic is
/// needed because of range and rounding issues, so it's on the type itself to implement it.

pub trait Mix : Color {
    /// Given two Colors, returns a Color representing their midpoint: usually, this means their
    /// midpoint in some projection into three-dimensional space.
    fn mix(self, other: Self) -> Self;
}

impl<T: Color + From<Coord> + Into<Coord>> Mix for T {
    /// Given two colors that represent the points (a1, b1, c1) and (a2, b2, c2) in some common
    /// projection, returns the color (a1 + a2, b1 + b2, c1 + c2) / 2.
    fn mix(self, other: T) -> T {
        // convert to 3D space, add, divide by 2, come back
        let c1: Coord = self.into();
        let c2: Coord = other.into();
        T::from((c1 + c2) / 2)
    }        
}

// `XYZColor` notably doesn't implement conversion to and from `Coord` because illuminant information
// can't be preserved: this means that mixing colors with different illuminants would produce
// incorrect results. The following custom implementation of the Mix trait fixes this by converting
// colors to the same gamut first.
impl Mix for XYZColor {
    /// Uses the current XYZ illuminant as the base, and uses the chromatic adapation transform that
    /// the `XYZColor` struct defines (as `color_adapt`).
    fn mix(self, other: XYZColor) -> XYZColor {
        // convert to same illuminant
        let other_c = other.color_adapt(self.illuminant);
        // now just take the midpoint in 3D space
        let c1: Coord = Coord{x: self.x, y: self.y, z: self.z};
        let c2: Coord = Coord{x: other_c.x, y: other_c.y, z: other_c.z};
        let mixed_coord = (c1 + c2) / 2.0;
        XYZColor{
            x: mixed_coord.x,
            y: mixed_coord.y,
            z: mixed_coord.z,
            illuminant: self.illuminant
        }
    }
}

impl Mix for RGBColor {
    fn mix(self, other: RGBColor) -> RGBColor {
        let (r1, g1, b1) = self.into();
        let (r2, g2, b2) = other.into();
        let (r, g, b) = (((r1 as u16 + r2 as u16) / 2) as u8,
                         ((g1 as u16 + g2 as u16) / 2) as u8,
                         ((b1 as u16 + b2 as u16) / 2) as u8);
        RGBColor{r, g, b}
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn can_display_colors() {
        let b = 128;
        for i in 0..8 {
            let mut line = String::from("");
            let r = i * 16;
            for j in 0..8 {
                let g = j * 16;
                line.push_str(RGBColor{r, g, b}.write_colored_str("■").as_str());                
            }
            println!("{}", line);        }
    }
    
    #[test]
    fn xyz_to_rgb() {
        let xyz = XYZColor{x: 0.41874, y: 0.21967, z: 0.05649, illuminant: Illuminant::D65};
        let rgb: RGBColor = xyz.convert();
        assert_eq!(rgb.r, 254);
        assert_eq!(rgb.g, 23);
        assert_eq!(rgb.b, 55);
    }

    #[test]
    fn rgb_to_xyz() {
        let rgb = RGBColor{r: 45, g: 28, b: 156};
        let xyz: XYZColor = rgb.to_xyz(Illuminant::D65);
        // these won't match exactly cuz floats, so I just check within a margin
        assert!((xyz.x - 0.0750).abs() <= 0.01);
        assert!((xyz.y - 0.0379).abs() <= 0.01);
        assert!((xyz.z-  0.3178).abs() <= 0.01);
    }
    // for now, not gonna use since the fun color adaptation demo already runs this
    #[allow(dead_code)]
    fn test_xyz_color_display() {
        println!();
        let y = 0.5;
        for i in 0..21 {
            let mut line = String::from("");
            for j in 0..21 {
                let x = i as f64 * 0.8 / 20.0;
                let z = j as f64 * 0.8 / 20.0;
                line.push_str(XYZColor{x, y, z, illuminant: Illuminant::D65}.write_colored_str("■").as_str());
            }

            println!("{}", line);
        }
    }
    #[test]
    fn test_rgb_to_string() {
        let c1 = RGBColor{r: 0, g: 0, b: 0};
        let c2 = RGBColor{r: 244, g: 182, b: 33};
        let c3 = RGBColor{r: 0, g: 255, b: 0};
        assert_eq!(c1.to_string(), "#000000");
        assert_eq!(c2.to_string(), "#F4B621");
        assert_eq!(c3.to_string(), "#00FF00");
    }
    #[test]
    fn test_mix_rgb() {
        let c1 = RGBColor::from((0, 0, 255));
        let c2 = RGBColor::from((255, 0, 1));
        let c3 = RGBColor::from((127, 7, 19));
        assert_eq!(c1.mix(c2).to_string(), "#7F0080");
        assert_eq!(c1.mix(c3).to_string(), "#3F0389");
        assert_eq!(c2.mix(c3).to_string(), "#BF030A");
    }
    #[test]
    fn test_mix_xyz() {
        // note how I'm using fractions with powers of 2 in the denominator to prevent floating-point issues
        let c1 = XYZColor{x: 0.5, y: 0.25, z: 0.75, illuminant: Illuminant::D65};
        let c2 = XYZColor{x: 0.625, y: 0.375, z: 0.5, illuminant: Illuminant::D65};
        let c3 = XYZColor{x: 0.75, y: 0.5, z: 0.25, illuminant: Illuminant::D65};
        assert_eq!(c1.mix(c3), c2);
        assert_eq!(c3.mix(c1), c2);
    }
    #[test]
    fn test_xyz_color_adaptation() {
        // I can literally not find a single API or something that does this so I can check the
        // values, so I'll just hope that it's good enough to check that converting between several
        // illuminants and back again gets something good
        let c1 = XYZColor{x: 0.5, y: 0.75, z: 0.6, illuminant: Illuminant::D65};
        let c2 = c1.color_adapt(Illuminant::D50).color_adapt(Illuminant::D55);
        let c3 = c1.color_adapt(Illuminant::D75).color_adapt(Illuminant::D55);
        assert!((c3.x - c2.x).abs() <= 0.01);
        assert!((c3.y - c2.y).abs() <= 0.01);
        assert!((c3.z - c2.z).abs() <= 0.01);
    }
    #[test]
    fn test_chromatic_adapation_to_same_light() {
        let xyz = XYZColor{x: 0.4, y: 0.6, z: 0.2, illuminant: Illuminant::D65};
        let xyz2 = xyz.color_adapt(Illuminant::D65);
        assert_eq!(xyz, xyz2);
    }
    #[test]
    fn fun_color_adaptation_demo() {
        println!();
        let w: usize = 120;
        let h: usize = 60;
        let d50_wp = Illuminant::D50.white_point();
        let d75_wp = Illuminant::D75.white_point();
        let d50 = XYZColor{x: d50_wp[0], y: d50_wp[1], z: d50_wp[2],
                           illuminant:Illuminant::D65};
        let d75 = XYZColor{x: d75_wp[0], y: d75_wp[1], z: d75_wp[2],
                           illuminant:Illuminant::D65};
        for _ in 0..h+1 {
            println!("{}{}", d50.write_color().repeat(w / 2), d75.write_color().repeat(w / 2));
        }
        
        println!();
        println!();
        let y = 0.5;
        println!();
        for i in 0..(h+1) {
            let mut line = String::from("");
            let x = i as f64 * 0.9 / h as f64;
            for j in 0..(w / 2) {
                let z = j as f64 * 0.9 / w as f64;
                line.push_str(XYZColor{x, y, z, illuminant: Illuminant::D50}.write_color().as_str());
            }
            for j in (w / 2)..(w+1) {
                let z = j as f64 * 0.9 / w as f64;
                line.push_str(XYZColor{x, y, z, illuminant: Illuminant::D75}.write_color().as_str());
            }
            println!("{}", line);
        }
        println!();
        println!();
        for i in 0..(h+1) {
            let mut line = String::from("");
            let x = i as f64 * 0.9 / h as f64;
            for j in 0..w {
                let z = j as f64 * 0.9 / w as f64;
                line.push_str(XYZColor{x, y, z, illuminant: Illuminant::D65}.write_color().as_str());
            }
            println!("{}", line);
        }
    }
    #[test]
    fn test_rgb_from_hex() {
        // test rgb format
        let rgb = RGBColor::from_hex_code("#172844").unwrap();
        assert_eq!(rgb.r, 23);
        assert_eq!(rgb.g, 40);
        assert_eq!(rgb.b, 68);
        // test with letters and no hex
        let rgb = RGBColor::from_hex_code("a1F1dB").unwrap();
        assert_eq!(rgb.r, 161);
        assert_eq!(rgb.g, 241);
        assert_eq!(rgb.b, 219);
        // test for error if 7 chars
        let rgb = RGBColor::from_hex_code("#1244444");
        assert!(match rgb {
            Err(x) if x == RGBParseError::InvalidHexSyntax => true,
            _ => false
        });
        // test for error if invalid hex chars
        let rgb = RGBColor::from_hex_code("#ffggbb");
        assert!(match rgb {
            Err(x) if x == RGBParseError::InvalidHexSyntax => true,
            _ => false
        });               
    }
}
