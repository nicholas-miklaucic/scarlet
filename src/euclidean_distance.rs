//! This file provides a trait and a default implementation for most standard coordinate systems that
//! implements the standard idea of **Euclidean distance**: treating two colors as points in 3D space,
//! and returning the length of the line between them. Note that this is not perceptually accurate
//! (i.e., what humans would describe as colors looking different) in any color space. In fact, this
//! may have extremely bizarre results: in HSV, for example, two blacks with different hue values
//! might be considered as different despite their complete visual equivalence.To get the perceptual
//! analog of distance, use the distance() method of Color. To implement this trait, it is only
//! necessary to implement Color and Into<Coord>: already what Mix and other such traits require.

use coord::Coord;
use color::Color;

pub trait EuclideanDistance : Color + Into<Coord> {
    /// Gets the Euclidean distance between these two points when embedded in 3D space. Formally
    /// speaking, this is a *metric*: it is 0 if and only if self and other are the same, 
    fn euclidean_distance(self, other: Self) -> f64 {
        let c1: Coord = self.into();
        let c2: Coord = other.into();
        c1.euclidean_distance(&c2)
    }
}

impl<T: Color + Into<Coord>> EuclideanDistance for T {
    // nothing to do
}


#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use colors::cielabcolor::CIELABColor;

    #[test]
    fn test_cielab_distance() {
        // pretty much should work the same for any type, so why not just CIELAB?
        let lab1 = CIELABColor{l: 10.5, a: -45.0, b: 40.0};
        let lab2 = CIELABColor{l: 54.2, a: 65.0, b: 100.0};
        println!("{}", lab1.euclidean_distance(lab2));
        assert!((lab1.euclidean_distance(lab2) - 132.70150715).abs() <= 1e-7);
    }
}
