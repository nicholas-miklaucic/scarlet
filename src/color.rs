/// This file defines the Color trait and all of the standard color types that implement it.

use std::convert::From;
use std::string::ToString;
extern crate termion;
use self::termion::color::{Fg, Reset, Rgb};
use super::coord::Coord;


/// A point in the CIE 1931 XYZ color space.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct XYZColor {
    // these need to all be positive
    // TODO: way of implementing this constraint in code?
    x: f64,
    y: f64,
    z: f64,
    // TODO: deal with illuminant
}

impl From<Coord> for XYZColor {
    fn from(nums: Coord) -> Self {
        XYZColor{x: nums.x, y: nums.y, z: nums.z}
    }
}

impl Into<Coord> for XYZColor {
    fn into(self) -> Coord {
        Coord {
            x: self.x,
            y: self.y,
            z: self.z
        }
    }
}


/// A trait that includes any color representation that can be converted to and from the CIE 1931 XYZ
/// color space.
pub trait Color {
    fn from_xyz(XYZColor) -> Self;
    fn to_xyz(&self) -> XYZColor;

    fn convert<T: Color>(&self) -> T {
        T::from_xyz(self.to_xyz())
    }
    fn write_colored_str(&self, text: &str) -> String {
        let rgb: RGBColor = self.convert();
        rgb.base_write_colored_str(text)
    }
}

impl Color for XYZColor {
    fn from_xyz(xyz: XYZColor) -> XYZColor {
        xyz
    }
    fn to_xyz(&self) -> XYZColor {
        *self
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
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
        // TODO: implement full illuminant list from
        // https://github.com/hughsie/colord/tree/master/data/illuminant
        // and deal with observers

        // first, get linear RGB values (i.e., without gamma correction)
        // https://en.wikipedia.org/wiki/SRGB#Specification_of_the_transformation

        // note how the diagonals are large: X, Y, Z, roughly equivalent to R, G, B
        let rgb_lin_vec = vec![3.2406 * xyz.x - 1.5372 * xyz.y - 0.4986 * xyz.z,
                               -0.9689 * xyz.x + 1.8758 * xyz.y + 0.0415 * xyz.z,
                               0.0557 * xyz.x - 0.2040 * xyz.y + 1.0570 * xyz.z];
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

    fn to_xyz(&self) -> XYZColor {
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

        XYZColor{x, y, z}
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
        T::from((c1 + c2) / 2u8)
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
        let xyz = XYZColor{x: 0.41874, y: 0.21967, z: 0.05649};
        let rgb: RGBColor = xyz.convert();
        assert_eq!(rgb.r, 254);
        assert_eq!(rgb.g, 23);
        assert_eq!(rgb.b, 55);
    }

    #[test]
    fn rgb_to_xyz() {
        let rgb = RGBColor{r: 45, g: 28, b: 156};
        let xyz: XYZColor = rgb.to_xyz();
        // these won't match exactly cuz floats, so I just check within a margin
        assert!((xyz.x - 0.0750).abs() <= 0.01);
        assert!((xyz.y - 0.0379).abs() <= 0.01);
        assert!((xyz.z-  0.3178).abs() <= 0.01);
    }
    #[test]
    fn test_xyz_color_display() {
        println!();
        let y = 0.5;
        for i in 0..21 {
            let mut line = String::from("");
            for j in 0..21 {
                let x = i as f64 * 0.8 / 20.0;
                let z = j as f64 * 0.8 / 20.0;
                line.push_str(XYZColor{x, y, z}.write_colored_str("■").as_str());
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
        let c1 = XYZColor{x: 0.5, y: 0.25, z: 0.75};
        let c2 = XYZColor{x: 0.625, y: 0.375, z: 0.5};
        let c3 = XYZColor{x: 0.75, y: 0.5, z: 0.25};
        assert_eq!(c1.mix(c3), c2);
        assert_eq!(c3.mix(c1), c2);
    }
}
