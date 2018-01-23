/// This file defines the Color trait and all of the standard color types that implement it.


use std::convert;
use termion::color;

/// A point in the CIE 1931 XYZ color space.
pub struct XYZColor {
    // these need to all be positive
    // TODO: way of implementing this constraint in code?
    x: f64,
    y: f64,
    z: f64,
    illuminant: String  // TODO: deal with this more later
}


/// A trait that includes any color representation that can be converted to and from the CIE 1931 XYZ
/// color space.
pub trait Color {
    fn from_xyz(XYZColor) -> Self;
    fn to_xyz(&self) -> XYZColor;
    
}

