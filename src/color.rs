// Now, we begin the public part of this: defining the structs that we'll use to represent different
// colors in different color spaces.

// First, we define a trait that identifies a color as a color: for our purposes, this will be some
// way of converting to and from a color in the CIE XYZ space (along with an associated luminant).

use std::convert;
use termion::color;


pub struct XYZColor {
    // these need to all be positive
    // TODO: way of implementing this constraint in code?
    x: f64,
    y: f64,
    z: f64,
    illuminant: String  // TODO: deal with this more later
}


pub trait Color {
    fn from_xyz(XYZColor) -> Self;
    fn to_xyz(&self) -> XYZColor;
    
}

