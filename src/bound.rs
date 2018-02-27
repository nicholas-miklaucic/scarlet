//! This module describes the Bound trait, which allows for a description of what colors a color
//! gamut supports. For example, the sRGB gamut only supports RGB values ranging from 0-1 that are
//! scaled to 0-255, which is about 30% of the total visible range of human vision.


use color::{Color, RGBColor};
use colorpoint::ColorPoint;
use coord::Coord;


/// Describes a color space in which the total space of representable colors has explicit bounds
/// besides those imposed by human vision. For example, an sRGB color can't have negative values for
/// any of its components, whereas the CIELAB space can feasibly describe even those colors that
/// cannot be viewed by humans. This only applies to colors that can be embedded in 3D space, hence
/// the use of the ColorPoint trait as a dependency.
/// # Example
/// Bound a clearly-problematic color within sRGB.
///
/// ```
/// # use scarlet::prelude::*;
/// # use scarlet::colors::CIELABColor;
/// let out_of_bounds = CIELABColor{l: 1., a: 150., b: -150.};
/// let in_bounds: RGBColor = RGBColor::clamp(out_of_bounds).convert();
/// let still_in_bounds: RGBColor = RGBColor::clamp(in_bounds).convert();
/// assert_eq!(in_bounds.to_string(), still_in_bounds.to_string());
/// let in_bounds_lab: CIELABColor = in_bounds.convert();
/// println!("{} {} {}", in_bounds_lab.l, in_bounds_lab.a, in_bounds_lab.b);
/// // prints 27.024908432754984 64.48329922444846 -105.76675512389784
/// // notice difference from before: also, note how every component changes to find the closest match
/// ```
pub trait Bound : Color + ColorPoint {
    /// Returns an array [(min1, max1), (min2, max2), (min3, max3)] that represents the bounds on each
    /// component of the color space, in the order that they appear in the Coord representation. If
    /// some parts of the bounds don't exist, using infinity or negative infinity works.
    fn bounds() -> [(f64, f64); 3];
    /// Given a Coord, returns a Coord such that each component has been clamped to the correct
    /// bounds. See trait documentation for example usage.
    fn clamp_coord(point: Coord) -> Coord {
        let ranges = Self::bounds();
        let mut point_vals = [0.; 3];
        for i in 0..3 {
            let component = [point.x, point.y, point.z][i];
            let (min, max) = ranges[i];
            point_vals[i] = if component < min {
                min
            } else if component > max {
                max
            } else {
                component
            };
        }
        Coord {
            x: point_vals[0],
            y: point_vals[1],
            z: point_vals[2],
        }
    }
    /// Given a Color that can be embedded in 3D space, returns a new version of that color that is in
    /// the bounds of this color space, even if the coordinate systems of the two spaces differ. If
    /// the color is already in the gamut, it simply returns a copy. See trait documentation for
    /// example usage.
    fn clamp<T: ColorPoint>(color: T) -> T {
        let converted_color: Self = color.convert();
        let point: Coord = converted_color.into();
        Self::from(Self::clamp_coord(point)).convert()
    }
}

// implement Bound for the base colors in the color module, to avoid cluttering that more than it
// already is
impl Bound for RGBColor {
    fn bounds() -> [(f64, f64); 3] {
        [(0., 1.), (0., 1.), (0., 1.)]
    }
}


#[cfg(test)]
mod tests {
    use super::Bound;
    use color::Color;
    use color::RGBColor;
    use colors::hslcolor::HSLColor;
    use colors::hsvcolor::HSVColor;

    #[test]
    fn test_zero_one_bounds() {
        let color1 = RGBColor{r: 0.1, g: -0.2, b: 1.2};
        assert!(RGBColor::clamp(color1).visually_indistinguishable(&RGBColor{r: 0.1, g: 0., b: 1.}));
    }

    #[test]
    fn test_hue_bounds() {
        let color1 = HSLColor{h: -24.0, s: -0.2, l: 1.1};
        let color2 = HSVColor{h: 375.0, s: 0.2, v: 0.5};
        let color3 = HSVColor{h: 255.0, s: 0.6, v: 0.7};
        assert!(color3.visually_indistinguishable(&HSVColor::clamp(color3)));
        assert!(HSVColor::clamp(color2).visually_indistinguishable(&HSVColor {
            h: 360.,
            s: 0.2,
            v: 0.5,
        }));
        assert!(HSLColor::clamp(color1).visually_indistinguishable(&HSLColor {
            h: 0.,
            s: 0.,
            l: 1.
        }));
    }
}
