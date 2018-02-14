//! This file implements most of the standard color functions that essentially work on 3D space,
//! including Euclidean distance, midpoints, and more. All of these methods work on `Color` types
//! that implement `Into<Coord>` and `From<Coord>`, and some don't require `From<Coord>`. This makes
//! it easy to provide these for custom `Color` types.

use coord::Coord;
use color::{Color, XYZColor};
use colors::cieluvcolor::CIELUVColor;
use visual_gamut::read_cie_spectral_data;
use super::geo::{Closest, LineString, Point};
use super::geo::prelude::*;

/// Some errors that might pop up when dealing with colors as coordinates.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColorCalcError {
    MismatchedWeights,
}

/// A trait that indicates that the current Color can be embedded in 3D space. This also requires
/// `Clone` and `Copy`: there shouldn't be any necessary information outside of the coordinate data.
pub trait ColorPoint: Color + Into<Coord> + From<Coord> + Clone + Copy {
    /// Gets the Euclidean distance between these two points when embedded in 3D space. This should
    /// **not** be used as an analog of color similarity: use the `distance()` function for
    /// that. Formally speaking, this is a *metric*: it is 0 if and only if self and other are the
    /// same, the distance between two points A and B is never larger than the distance from A to C
    /// and the distance from B to C summed, and it is never negative.
    fn euclidean_distance(self, other: Self) -> f64 {
        let c1: Coord = self.into();
        let c2: Coord = other.into();
        c1.euclidean_distance(&c2)
    }

    /// Gets the *weighted midpoint* of two colors in a space as a new `Color`. This is defined as the
    /// color corresponding to the point along the line segment connecting the two points such that
    /// the distance to the second point is the weight, which for most applications needs to be
    /// between 0 and 1. For example, a weight of 0.9 would make the midpoint one-tenth as much
    /// affected by the second points as the first.
    fn weighted_midpoint(self, other: Self, weight: f64) -> Self {
        let c1: Coord = self.into();
        let c2: Coord = other.into();
        Self::from(c1.weighted_midpoint(&c2, weight))
    }

    /// Like `weighted_midpoint`, but with `weight = 0.5`: essentially, the `Color` representing the
    /// midpoint of the two inputs in 3D space.
    fn midpoint(self, other: Self) -> Self {
        let c1: Coord = self.into();
        let c2: Coord = other.into();
        Self::from(c1.midpoint(&c2))
    }

    /// Returns the weighted average of a given set of colors. Weights will be normalized so that they
    /// sum to 1. Each component of the final value will be calculated by summing the components of
    /// each of the input colors multiplied by their given weight.
    /// # Errors
    /// Returns `ColorCalcError::MismatchedWeights` if the number of colors (`self` and anything in
    /// `others`) and the number of weights mismatch.
    fn weighted_average(
        self,
        others: Vec<Self>,
        weights: Vec<f64>,
    ) -> Result<Self, ColorCalcError> {
        if others.len() + 1 != weights.len() {
            Err(ColorCalcError::MismatchedWeights)
        } else {
            let c1: Coord = self.into();
            let norm: f64 = weights.iter().sum();
            let mut coord = c1 * weights[0] / norm;
            for i in 1..weights.len() {
                coord = coord + others[i - 1].into() * weights[i] / norm;
            }
            Ok(Self::from(coord))
        }
    }
    /// Returns the arithmetic mean of a given set of colors. Equivalent to `weighted_average` in the
    /// case where each weight is the same.
    fn average(self, others: Vec<Self>) -> Coord {
        let c1: Coord = self.into();
        let other_cs = others.iter().map(|x| (*x).into()).collect();
        c1.average(other_cs)
    }

    /// Returns `true` if the color is outside the range of human vision. Uses the CIE 1931 standard
    /// observer spectral data.
    fn is_imaginary(&self) -> bool {
        let (_wavelengths, xyz_data) = read_cie_spectral_data();
        // convert to chromaticity coordinates
        // use the explicit formulae instead of CIELUVColor to reduce rounding errors
        // we only care about those coordinates
        let uv_func = |xyz: XYZColor| {
            let denom = xyz.x + 15.0 * xyz.y + 3.0 * xyz.z;
            (4.0 * xyz.x / denom, 9.0 * xyz.y / denom)
        };
        let self_uv: (f64, f64) = (uv_func(self.convert())).into();
        let uv_data: Vec<(f64, f64)> = xyz_data.into_iter().map(uv_func).collect();
        let self_point = Point::new(self_uv.0, self_uv.1);

        // this is an annoying algorithm, so I'm using a crate instead
        let line: LineString<f64> = uv_data.into();
        line.contains(&self_point)
    }

    /// Returns the closest color that can be seen by the human eye. If the color is not imaginary,
    /// returns itself.
    fn closest_real_color(&self) -> Self {
        // if real color, return itself
        if !self.is_imaginary() {
            *self
        } else {
            let (_wavelengths, xyz_data) = read_cie_spectral_data();
            // convert to chromaticity coordinates
            // use the explicit formulae instead of CIELUVColor to reduce rounding errors
            // we only care about those coordinates
            let uv_func = |xyz: XYZColor| {
                let denom = xyz.x + 15.0 * xyz.y + 3.0 * xyz.z;
                (4.0 * xyz.x / denom, 9.0 * xyz.y / denom)
            };
            // we need to keep luminance data to convert back, so we use CIELUV explicitly
            let mut self_luv: CIELUVColor = self.convert();
            let self_uv = (self_luv.u, self_luv.v);
            let uv_data: Vec<(f64, f64)> = xyz_data.into_iter().map(uv_func).collect();
            let self_point = Point::new(self_uv.0, self_uv.1);

            // this is also an annoying algorithm: just use the crate
            let line: LineString<f64> = uv_data.into();
            let closest_point = line.closest_point(&self_point);
            // convert back into original type
            match closest_point {
                Closest::Intersection(p) => {
                    self_luv.u = p.x();
                    self_luv.v = p.y();
                }
                Closest::SinglePoint(p) => {
                    self_luv.u = p.x();
                    self_luv.v = p.y();
                }
                Closest::Indeterminate => {
                    // should never happen
                    panic!("Indeterminate closest point! Please report this error");
                }
            }
            self_luv.convert()
        }
    }

    /// Returns a Vector of colors that starts with this color, ends with the given other color, and
    /// evenly transitions between colors. The given `n` is the number of additional colors to add.
    fn gradient_scale(&self, other: &Self, n: usize) -> Vec<Self> {
        let mut grad_scale = Vec::new();
        // n + 2 total colors: scale this range to [0, 1] inside the loop
        for i in 0..n + 2 {
            let weight = i as f64 / (n + 1) as f64;
            grad_scale.push((*other).weighted_midpoint(*self, weight));
        }
        grad_scale
    }

    /// Returns a pointer to a function that maps floating-point values from 0 to 1 to colors, such
    /// that 0 returns `self`, 1 returns `other`, and anything in between returns a mix (calculated
    /// linearly). Although it is possible to extrapolate outside of the range [0, 1], this is not
    /// a guarantee and may change without warning.
    ///
    /// # Examples
    /// ```rust
    /// use scarlet::color::RGBColor;
    /// use scarlet::color_funcs::ColorPoint;
    /// let start = RGBColor::from_hex_code("#11457c").unwrap();
    /// let end = RGBColor::from_hex_code("#774bdc").unwrap();
    /// let grad = start.gradient(&end);
    /// let color_at_start = grad(0.).to_string(); // #11457C
    /// let color_at_end = grad(1.).to_string(); // #774BDC
    /// let color_at_third = grad(2./6.).to_string(); // #33479C
    /// ```
    fn gradient(&self, other: &Self) -> Box<Fn(f64) -> Self> {
        let c1: Coord = (*self).into();
        let c2: Coord = (*other).into();
        println!("{:?}, {:?}", c1, c2);
        Box::new(move |x| Self::from(c2.weighted_midpoint(&c1, x)))
    }

    /// Returns a pointer to a function that maps floating-point values from 0 to 1 to colors, such
    /// that 0 returns `self`, 1 returns `other`, and anything in between returns a mix (calculated
    /// by the cube root of the given value). Although it is possible to extrapolate outside of the
    /// range [0, 1], this is not a guarantee and may change without warning.
    ///
    /// # Examples
    /// ```rust
    /// use scarlet::color::RGBColor;
    /// use scarlet::color_funcs::ColorPoint;
    /// let start = RGBColor::from_hex_code("#11457c").unwrap();
    /// let end = RGBColor::from_hex_code("#774bdc").unwrap();
    /// let grad = start.cbrt_gradient(&end);
    /// let color_at_start = grad(0.).to_string(); // #11457C
    /// let color_at_end = grad(1.).to_string(); // #774BDC
    /// let color_at_third = grad(2./6.).to_string(); // #5849BF
    /// ```
    fn cbrt_gradient(&self, other: &Self) -> Box<Fn(f64) -> Self> {
        let c1: Coord = (*self).into();
        let c2: Coord = (*other).into();
        println!("{:?}, {:?}", c1, c2);
        Box::new(move |x| Self::from(c2.weighted_midpoint(&c1, x.cbrt())))
    }
}

impl<T: Color + Into<Coord> + From<Coord> + Copy + Clone> ColorPoint for T {
    // nothing to do
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use colors::cielabcolor::CIELABColor;
    use color::RGBColor;

    #[test]
    fn test_cielab_distance() {
        // pretty much should work the same for any type, so why not just CIELAB?
        let lab1 = CIELABColor {
            l: 10.5,
            a: -45.0,
            b: 40.0,
        };
        let lab2 = CIELABColor {
            l: 54.2,
            a: 65.0,
            b: 100.0,
        };
        println!("{}", lab1.euclidean_distance(lab2));
        assert!((lab1.euclidean_distance(lab2) - 132.70150715).abs() <= 1e-7);
    }
    #[test]
    fn test_grad_scale() {
        let start = RGBColor::from_hex_code("#11457c").unwrap();
        let end = RGBColor::from_hex_code("#774bdc").unwrap();
        let grad_hexes: Vec<String> = start
            .gradient_scale(&end, 5)
            .iter()
            .map(|x| x.to_string())
            .collect();
        assert_eq!(
            grad_hexes,
            vec![
                "#11457C", "#22468C", "#33479C", "#4448AC", "#5549BC", "#664ACC", "#774BDC"
            ]
        );
    }
    #[test]
    fn test_grad_func() {
        let start = RGBColor::from_hex_code("#11457c").unwrap();
        let end = RGBColor::from_hex_code("#774bdc").unwrap();
        let grad = start.gradient(&end);
        assert_eq!(grad(1.).to_string(), "#774BDC");
        assert_eq!(grad(0.).to_string(), "#11457C");
        assert_eq!(grad(2. / 6.).to_string(), "#33479C");
    }
    #[test]
    fn test_cbrt_grad_func() {
        let start = RGBColor::from_hex_code("#11457c").unwrap();
        let end = RGBColor::from_hex_code("#774bdc").unwrap();
        let grad = start.cbrt_gradient(&end);
        assert_eq!(grad(1.).to_string(), "#774BDC");
        assert_eq!(grad(0.).to_string(), "#11457C");
        assert_eq!(grad(2. / 6.).to_string(), "#5849BF");
    }
}
