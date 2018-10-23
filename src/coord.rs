//! This module contains a struct, [`Coord`](coord::Coord), that models a 3D coordinate space and supports limited
//! math in 3 dimensions with scalars and other coordinates. Used to unify math with colors that is
//! the same, just with different projections into 3D space.

use std::ops::{Add, Div, Mul, Sub};
use num;
use num::{Num, NumCast};

/// Represents a scalar value that can be easily converted, described using the common numeric traits
/// in [`num`]. Anything that falls under this category can be multiplied by a [`Coord`] to scale
/// it. This has no added functionality: it's just for convenience.
pub trait Scalar: NumCast + Num {}

impl<T: NumCast + Num> Scalar for T {}

/// A point in 3D space. Supports many common arithmetic operations on points.
/// `Coord` has three axes, denoted `x`, `y`, and `z`. These are not any different in any method of
/// `Coord`, so the distinction between them is completely conventional. In Scarlet, any [`Color`]
/// that converts to and from a `Coord` will match its components with these axes in the order of the
/// letters in its name: for example, `CIELABColor` maps to a coordinate such that `l` is on the
/// x-axis, `a` is on the y-axis, and `b` is on the z-axis.
///
/// # Examples
/// ## Basic Operations
/// ```
/// # use scarlet::coord::Coord;
/// let point_1 = Coord{x: 1., y: 8., z: 7.};
/// let point_2 = Coord{x: 7., y: 2., z: 3.};
/// // Add two points together to do componentwise addition.
/// let sum = point_1 + point_2;  // the point (8, 10, 10)
/// // Subtract two points the same way.
/// let diff = point_1 - point_2;  // the point (-6, 6, 4)
/// // There is no multiplication of two points, because there are many different ways to conceptualize
/// // multiplying two points and Scarlet doesn't need it. Instead, it supports scalar multiplication
/// // and division. This has the unfortunate side effect of not allowing multiplication one way.
/// let prod = point_1 * 2u8; // the point (2, 16, 14)
/// // switching the above operands' order would cause an error!
/// let quot = point_1 / 2.; // the point (0.5, 4, 3.5)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct Coord {
    /// The first axis.
    pub x: f64,
    /// The second axis.
    pub y: f64,
    /// The third axis.
    pub z: f64,
}

// Now we implement addition and subtraction, as well as division and multiplication by scalars. Note
// that because the multiplication of pnoints by points in 3D space has different defintions, we won't
// implement it: it's unclear what even the return type should be.
impl Add for Coord {
    type Output = Coord;
    fn add(self, rhs: Coord) -> Coord {
        return Coord {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        };
    }
}

/// This is a perfect analogue to numbers: for any Coords c1, c2, and c3 with the same type, c1 + c2 =
/// c3 implies c3 - c2 = c1 and c3 - c1 = c2, down to floating point error if that exists.
impl Sub for Coord {
    type Output = Coord;
    fn sub(self, rhs: Coord) -> Coord {
        return Coord {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        };
    }
}

// This implements basic scalar multiplication and division: (a, b, c) * s = (sa, sb, sc) and
// similarly for division. This is unfortunately not commutative, but it'll do.
impl<U: Scalar> Mul<U> for Coord {
    type Output = Coord;
    fn mul(self, rhs: U) -> Coord {
        let r: f64 = num::cast(rhs).unwrap();
        return Coord {
            x: self.x * r,
            y: self.y * r,
            z: self.z * r,
        };
    }
}

impl<U: Scalar> Div<U> for Coord {
    type Output = Coord;
    fn div(self, rhs: U) -> Coord {
        if rhs.is_zero() {
            panic!("Division by 0!");
        } else {
            let r: f64 = num::cast(rhs).unwrap();
            Coord {
                x: self.x / r,
                y: self.y / r,
                z: self.z / r,
            }
        }
    }
}

// this will mostly be math stuff for colors
impl Coord {
    /// The midpoint between two 3D points: returns a new Coord.
    /// # Example
    /// ```
    /// # use scarlet::coord::Coord;
    /// let point1 = Coord{x: 0.25, y: 0., z: 1.};
    /// let point2 = Coord{x: 0.75, y: 1., z: 1.};
    /// let mid = point1.midpoint(&point2);
    /// assert!((mid.x - 0.5).abs() <= 1e-10);
    /// assert!((mid.y - 0.5).abs() <= 1e-10);
    /// assert!((mid.z - 1.).abs() <= 1e-10);
    /// ```
    pub fn midpoint(&self, other: &Coord) -> Coord {
        Coord {
            x: (&self.x + &other.x) / 2.0,
            y: (&self.y + &other.y) / 2.0,
            z: (&self.z + &other.z) / 2.0,
        }
    }
    /// The weighted midpoint: like the midpoint, but with weighted averages instead of the arithmetic
    /// mean. Very strange things may happen if the weight is not between 0 and 1. Note that a small
    /// weight moves values further away from the first point (the one calling the method), while a
    /// larger weight moves values away from the second point (the one being passed in).
    /// # Example
    /// ```
    /// # use scarlet::coord::Coord;
    /// let point1 = Coord{x: 0.2, y: 0., z: 1.};
    /// let point2 = Coord{x: 1., y: 0.8, z: 1.};
    /// let mid = point1.weighted_midpoint(&point2, 0.25);
    /// // note how this is not 0.6 because the weight has shifted it towards the second point
    /// assert!((mid.x - 0.8).abs() <= 1e-10);
    /// assert!((mid.y - 0.6).abs() <= 1e-10);
    /// assert!((mid.z - 1.).abs() <= 1e-10);
    /// ```
    pub fn weighted_midpoint(&self, other: &Coord, weight: f64) -> Coord {
        Coord {
            x: (&self.x * weight + (1.0 - weight) * &other.x),
            y: (&self.y * weight + (1.0 - weight) * &other.y),
            z: (&self.z * weight + (1.0 - weight) * &other.z),
        }
    }
    /// The Euclidean difference between two 3D points, defined as the square root of the sum of
    /// squares of differences in each axis.
    /// It's very tempting to use this is as an analogue for perceptual difference between two colors,
    /// but this should generally be avoided. The reason is that projection into 3D space does not
    /// necessarily make distance a good analogue of perceptual difference. A very clear example would
    /// be the two [`HSVColor`] points (360., 1., 1.) and (0., 1., 1.), which are the same point even
    /// though their difference is 360, or examples with very low luminance: (275., 0., 0.,) and
    /// (300., 0.4, 0.) represent the exact same color as well. Even in additive primary spaces like
    /// RGB, this is usually a bad way of getting color distance: for example, humans are very good at
    /// distinguishing between blues compared to greens, so two greens with the same Euclidean
    /// distance as two blues will look much closer. If you want a method of determining how different
    /// two colors look, use the [`color::distance`] method, which provides the current industry and
    /// scientific standard for doing so.
    ///
    /// [`HSVColor`]: ../colors/hsvcolor/struct.HSVColor.html
    /// [`color::distance`]: ../color/trait.Color.html#method.distance
    /// # Example
    /// ```
    /// # use scarlet::coord::Coord;
    /// let point1 = Coord{x: 0., y: 0., z: -1.};
    /// let point2 = Coord{x: 2., y: 3., z: 5.};
    /// let dist = point1.euclidean_distance(&point2);
    /// assert!((dist - 7.).abs() <= 1e-10);
    /// ```
    pub fn euclidean_distance(&self, other: &Coord) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2))
            .sqrt()
    }
    /// Gets the arithmetic mean of `self`, alongside other coordinates.
    /// # Example
    /// ```
    /// # use scarlet::coord::Coord;
    /// let point1 = Coord{x: 0., y: 0., z: 1.};
    /// let others = vec![Coord{x: 1., y: 1., z: 1.}, Coord{x: 2., y: 1., z: 1.}];
    /// let mean = point1.average(others);
    /// assert!((mean.x - 1.).abs() <= 1e-10);
    /// assert!((mean.y - 2. / 3.).abs() <= 1e-10);
    /// assert!((mean.z - 1.).abs() <= 1e-10);
    pub fn average(self, others: Vec<Coord>) -> Coord {
        let n = others.len() + 1;
        others.iter().fold(self, |x, y| x + *y) / n
    }
}
