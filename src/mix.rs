/// Describes a Color that can be mixed with other colors in its own 3D space. Mixing, in this
/// context, is taking the midpoint of two color projections in some space, or something consistent
/// with that idea: if colors A and B mix to A, that should mean B is the same as A, for
/// example. Although this is not currently the case, note that this implies that the gamut of this
/// Color is convex: any two Colors of the same type may be mixed to form a third valid one.

/// Note that there is one very crucial thing to remember about mixing: it differs depending on the
/// color space being used. For example, if there are two colors A and B, A.mix(B) may produce very
/// different results than A_conv.mix(B_conv) if A_conv and B_conv are the results of A.convert() and
/// B.convert(). For this reason, A.mix(B) is only allowed if A and B share a type: otherwise,
/// A.mix(B) could be different than B.mix(A), which is error-prone and unintuitive.

/// There is a default implementation for Colors that can interconvert to Coord. This helps ensure
/// that the most basic case functions appropriately. For any other type of Color, special logic is
/// needed because of range and rounding issues, so it's on the type itself to implement it.

/// Especially note that color mixing as one thinks of with paints or other subtractive mixtures will
/// almost definitely not agree with the output of Scarlet, because computer monitors use additive
/// mixing while pigments use subtractive mixing. Yellow mixed with blue in most RGB or other systems
/// is gray, not green.

use color::{Color, XYZColor};
use coord::Coord;


pub trait Mix: Color {
    /// Given two Colors, returns a Color representing their midpoint: usually, this means their
    /// midpoint in some projection into three-dimensional space.
    fn mix(self, other: Self) -> Self;
}

impl<T: Color + From<Coord> + Into<Coord>> Mix for T {
    /// Given two colors that represent the points (a1, b1, c1) and (a2, b2, c2) in some common
    /// projection, returns the color (a1 + a2, b1 + b2, c1 + c2) / 2.
    fn mix(self, other: T) -> T {
        // convert to 3D space, add, divide by 2, come back
        let c1: Coord = self.into();
        let c2: Coord = other.into();
        T::from(c1.midpoint(&c2))
    }
}

// `XYZColor` notably doesn't implement conversion to and from `Coord` because illuminant information
// can't be preserved: this means that mixing colors with different illuminants would produce
// incorrect results. The following custom implementation of the Mix trait fixes this by converting
// colors to the same gamut first.
impl Mix for XYZColor {
    /// Uses the current XYZ illuminant as the base, and uses the chromatic adapation transform that
    /// the `XYZColor` struct defines (as `color_adapt`).
    fn mix(self, other: XYZColor) -> XYZColor {
        // convert to same illuminant
        let other_c = other.color_adapt(self.illuminant);
        // now just take the midpoint in 3D space
        let c1: Coord = Coord {
            x: self.x,
            y: self.y,
            z: self.z,
        };
        let c2: Coord = Coord {
            x: other_c.x,
            y: other_c.y,
            z: other_c.z,
        };
        let mixed_coord = (c1 + c2) / 2.0;
        XYZColor {
            x: mixed_coord.x,
            y: mixed_coord.y,
            z: mixed_coord.z,
            illuminant: self.illuminant,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use color::RGBColor;
    use illuminants::Illuminant;
    #[test]
    fn test_mix_rgb() {
        let c1 = RGBColor::from((0, 0, 255));
        let c2 = RGBColor::from((255, 0, 1));
        let c3 = RGBColor::from((127, 7, 19));
        // testing rounding away from 0
        assert_eq!(c1.mix(c2).to_string(), "#800080");
        assert_eq!(c1.mix(c3).to_string(), "#400489");
        assert_eq!(c2.mix(c3).to_string(), "#BF040A");
    }
    #[test]
    fn test_mix_xyz() {
        // note how I'm using fractions with powers of 2 in the denominator to prevent floating-point issues
        let c1 = XYZColor {
            x: 0.5,
            y: 0.25,
            z: 0.75,
            illuminant: Illuminant::D65,
        };
        let c2 = XYZColor {
            x: 0.625,
            y: 0.375,
            z: 0.5,
            illuminant: Illuminant::D65,
        };
        let c3 = XYZColor {
            x: 0.75,
            y: 0.5,
            z: 0.25,
            illuminant: Illuminant::D65,
        };
        assert_eq!(c1.mix(c3), c2);
        assert_eq!(c3.mix(c1), c2);
    }
}
