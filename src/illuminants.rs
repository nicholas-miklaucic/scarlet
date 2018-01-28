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
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Illuminant {
    D50,
    D55,
    D65,
    D75,
}


/// A table of white point values for various CIE illuminants. As there are currently no static
/// HashMaps or the like in Rust, this is simply an array of arrays. The order of the rows is the
/// order of the Illuminant enum definition, which should be alphabetical and low-high in that
/// order. Each white point is an array of 3 `f64` values X, Y, and Z, normalized so that Y is 100.
pub static ILLUMINANT_WHITE_POINTS: [[f64; 3]; 4] = [
    [96.422, 100.000, 82.521],
    [95.682, 100.000, 92.129],
    [95.047, 100.000, 108.884],
    [94.972, 100.000, 122.638]
];
