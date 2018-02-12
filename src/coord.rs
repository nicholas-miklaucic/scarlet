/// This file contains a struct that models a 3D coordinate space and supports limited math in 3
/// dimensions with scalars and other Coordinates. Used to unify math with colors that is the same,
/// just with different projections into 3D space.

use std::ops::{Add, Div, Mul, Sub};
extern crate num;
use self::num::{Num, NumCast};

pub trait Scalar: NumCast + Num {}

impl<T: NumCast + Num> Scalar for T {}

/// A point in 3D space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coord {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Now we implement addition and subtraction, as well as division and multiplication by scalars. Note
/// that because the multiplication of pnoints by points in 3D space has different defintions, we won't
/// implement it: it's unclear what even the return type should be.
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

/// This implements basic scalar multiplication and division: (a, b, c) * s = (sa, sb, sc) and
/// similarly for division. This is unfortunately not commutative, but it'll do.
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
    pub fn midpoint(&self, other: &Coord) -> Coord {
        Coord {
            x: (&self.x + &other.x) / 2.0,
            y: (&self.y + &other.y) / 2.0,
            z: (&self.z + &other.z) / 2.0,
        }
    }
    /// The weighted midpoint: like midpoint, but with weighted averages instead of the arithmetic
    /// mean. Very strange things may happen if the weight is not between 0 and 1.
    pub fn weighted_midpoint(&self, other: &Coord, weight: f64) -> Coord {
        Coord {
            x: (&self.x * weight + (1.0 - weight) * &other.x),
            y: (&self.y * weight + (1.0 - weight) * &other.y),
            z: (&self.z * weight + (1.0 - weight) * &other.z),
        }
    }
    /// The Euclidean difference between two 3D points, defined as the square root of the sum of
    /// squares of differences in each axis.
    pub fn euclidean_distance(&self, other: &Coord) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2))
            .sqrt()
    }
    /// Gets the arithmetic mean of `self`, alongside other coordinates.
    pub fn average(self, others: Vec<Coord>) -> Coord {
        let n = others.len() + 1;
        others.iter().fold(self, |x, y| x + *y) / n
    }
}
