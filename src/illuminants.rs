//! This module provides an enum of various illuminants supported by Scarlet, as well as a table of
//! white point values for various CIE illuminants. The source for this table is from the [ASTM E308
//! standard](https://www.astm.org/Standards/E308.htm). The only one I could find available freely was
//! the outdated E308-01 standard, but these values should be the same: they're both copied
//! photographically from the CIE standard itself. These are normalized so that the Y (luminance)
//! value is 100. 

/// A listing of the supported CIE standard illuminants, standards that describe a particular set of
/// lighting conditions. The most common ones for computers are D50 and D65, differing kinds of
/// daylight. Other ones may be added as time goes on in a backwards-compatible manner.
// TODO: what illuminants to support?
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Illuminant {
    D50,
    D55,
    D65,
    D75,
    /// Represents a light of any given hue, as an array [X, Y, Z] in CIE 1931 space.
    Custom([f64; 3])
}


/// An array of illuminants, in alphabetical and numerical order (the same order as below).
pub static ILLUMINANTS: [Illuminant; 4] = [
    Illuminant::D50,
    Illuminant::D55,
    Illuminant::D65,
    Illuminant::D75,
];

/// A table of white point values for various CIE illuminants. As there are currently no static
/// HashMaps or the like in Rust, this is simply an array of arrays. The order of the rows is the
/// order of the Illuminant enum definition, which should be alphabetical and low-high in that
/// order. Each white point is an array of 3 `f64` values X, Y, and Z, normalized so that Y is 1.
pub static ILLUMINANT_WHITE_POINTS: [[f64; 3]; 4] = [
    [0.96422, 1.00000, 0.82521],
    [0.95682, 1.00000, 0.92129],
    [0.95047, 1.00000, 1.08884],
    [0.94972, 1.00000, 1.22638]
];

impl Illuminant {
    /// Gets the XYZ coordinates of the white point value of the illuminant, normalized so Y = 1.
    pub fn white_point(&self) -> [f64; 3] {
        match *self {
            Illuminant::D50 => ILLUMINANT_WHITE_POINTS[0],
            Illuminant::D55 => ILLUMINANT_WHITE_POINTS[1],
            Illuminant::D65 => ILLUMINANT_WHITE_POINTS[2],
            Illuminant::D75 => ILLUMINANT_WHITE_POINTS[3],
            Illuminant::Custom(xyz) => [
                xyz[0] / xyz[1],
                1.0,
                xyz[2] / xyz[1],
            ]
        }
    }
}

// TODO: tests?
