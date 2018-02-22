//! This module provides an enum of various illuminants supported by Scarlet, as well as their white
//! point values.The source for this table is from the [ASTM E308
//! standard](https://www.astm.org/Standards/E308.htm). The only one I could find available freely was
//! the outdated E308-01 standard, but these values should be the same: they're both copied
//! photographically from the CIE standard itself. These are normalized so that the Y (luminance)
//! value is 100.

/// A listing of the supported CIE standard illuminants, standards that describe a particular set of
/// lighting conditions. The most common ones for computers are D50 and D65, differing kinds of
/// daylight. Other ones may be added as time goes on, but they won't be removed and backwards
/// compatibility won't break without warning.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Illuminant {
    /// The CIE D50 standard illuminant. See [this
    /// page](https://en.wikipedia.org/wiki/Standard_illuminant#Illuminant_series_D) for more
    /// information. This has a rough color temperature of 5000 K, so it looks the reddest out of all
    /// these standard illuminants, matching "horizon light" in eastern North America. Scarlet uses
    /// D50 for internal conversions, as many color spaces use it to define their viewing environment.
    D50,
    /// The CIE D55 illuminant, slightly less red than D50. This is rather uncommon as a choice for
    /// most work, but is still fairly widely used.
    D55,
    /// The CIE D65 illuminant, representing average noon daylight in eastern North America. This is
    /// the recommended official standard for "representative daylight" according to the CIE. The most
    /// common RGB standard, on which you're probably reading this, assumes D65 as viewing conditions.
    D65,
    /// The CIE D75 illuminant. Rarer than the others, this is nontheless included for the occasional
    /// place where it might be used.
    D75,
    /// Represents a light of any given hue, as an array [X, Y, Z] in CIE 1931 space. This does not
    /// allow one to replicate any illuminant, but it does allow for custom illuminants and the
    /// ability to chromatically adapt to unique lighting conditions, like dark shade or colored light.
    Custom([f64; 3]),
}

/// A table of white point values for various CIE illuminants. As there are currently no static
/// HashMaps or the like in Rust, this is simply an array of arrays. The order of the rows is the
/// order of the Illuminant enum definition, which should be alphabetical and low-high in that
/// order. Each white point is an array of 3 `f64` values X, Y, and Z, normalized so that Y is 1.
pub(crate) static ILLUMINANT_WHITE_POINTS: [[f64; 3]; 4] = [
    [0.96422, 1.00000, 0.82521],
    [0.95682, 1.00000, 0.92129],
    [0.95047, 1.00000, 1.08884],
    [0.94972, 1.00000, 1.22638],
];

impl Illuminant {
    /// Gets the XYZ coordinates of the white point value of the illuminant, normalized so Y = 1.
    /// # Example
    /// # use scarlet::Illuminant;
    /// let wp = Illuminant::D65.white_point(); // [0.95047, 1.00000, 1.08884]
    /// assert!((wp[0] - 0.95047).abs() <= 1e-10);
    /// assert!((wp[1] - 1.00000).abs() <= 1e-10);
    /// assert!((wp[2] - 1.08884).abs() <= 1e-10);
    pub fn white_point(&self) -> [f64; 3] {
        match *self {
            Illuminant::D50 => ILLUMINANT_WHITE_POINTS[0],
            Illuminant::D55 => ILLUMINANT_WHITE_POINTS[1],
            Illuminant::D65 => ILLUMINANT_WHITE_POINTS[2],
            Illuminant::D75 => ILLUMINANT_WHITE_POINTS[3],
            Illuminant::Custom(xyz) => [xyz[0] / xyz[1], 1.0, xyz[2] / xyz[1]],
        }
    }
}
