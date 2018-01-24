/// This file defines the Color trait and all of the standard color types that implement it.

extern crate termion;
use self::termion::color::{Fg, Reset, Rgb};


/// A point in the CIE 1931 XYZ color space.
#[derive(Debug, Copy, Clone)]
pub struct XYZColor {
    // these need to all be positive
    // TODO: way of implementing this constraint in code?
    x: f64,
    y: f64,
    z: f64,
    illuminant: u8  // TODO: deal with this more later
}


/// A trait that includes any color representation that can be converted to and from the CIE 1931 XYZ
/// color space.
pub trait Color {
    fn from_xyz(XYZColor) -> Self;
    fn into_xyz(&self) -> XYZColor;

    fn convert<T: Color>(&self) -> T {
        T::from_xyz(self.into_xyz())
    }
}

impl Color for XYZColor {
    fn from_xyz(xyz: XYZColor) -> XYZColor {
        xyz
    }
    fn into_xyz(&self) -> XYZColor {
        *self
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RGBColor {
    r: u8,
    g: u8,
    b: u8,
    // TODO: add exact unclamped versions of each of these
}
    
impl RGBColor {
    /// Given a string, returns that string wrapped in codes that will color the foreground.
    pub fn write_colored_str(&self, text: &str) -> String {
        format!("{code}{text}{reset}",
                code=Fg(Rgb(self.r, self.g, self.b)),
                text=text,
                reset=Fg(Reset)
        )
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
        let rgb:Vec<u8> = float_vec.iter().map(|x| (*x * 255.0).round() as u8).collect();       
        RGBColor {
            r: rgb[0],
            g: rgb[1],
            b: rgb[2]
        }
    }

    fn into_xyz(&self) -> XYZColor {
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

        XYZColor{x, y, z, illuminant: 0}
    }
}
            
mod tests {
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
        let xyz = XYZColor{x: 0.41874, y: 0.21967, z: 0.05649, illuminant: 0};
        let rgb: RGBColor = xyz.convert();
        assert_eq!(rgb.r, 254);
        assert_eq!(rgb.g, 23);
        assert_eq!(rgb.b, 55);
    }

    #[test]
    fn rgb_to_xyz() {
        let rgb = RGBColor{r: 45, g: 28, b: 156};
        let xyz: XYZColor = rgb.into_xyz();
        // these won't match exactly cuz floats, so I just check within a margin
        assert!((xyz.x - 0.0750).abs() <= 0.01);
        assert!((xyz.y - 0.0379).abs() <= 0.01);
        assert!((xyz.z-  0.3178).abs() <= 0.01);
    }
}
