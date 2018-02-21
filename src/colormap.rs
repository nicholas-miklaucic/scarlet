//! This module defines a generalized trait for a colormap—a mapping of the numbers between 0 and 1
//! to colors in a continuous way—and provides some common ones used in programs like MATLAB and in
//! data visualization everywhere.


use std::iter::Iterator;
use color::{Color, RGBColor};
use colorpoint::ColorPoint;
use coord::Coord;
use matplotlib_cmaps;


/// A trait that models a colormap, a continuous mapping of the numbers between 0 and 1 to
/// colors. Any color output format is supported, but it must be consistent.
pub trait ColorMap<T: Color + Sized> {
    /// Maps a given number between 0 and 1 to a given output Color. This should never fail or panic
    /// except for NaN and similar: there should be some Color that marks out-of-range data.
    fn transform_single(&self, f64) -> T;
    /// Maps a given collection of numbers between 0 and 1 to an iterator of Colors. Does not evaluate
    /// lazily, because the colormap could have some sort of state that changes between iterations otherwise.
    fn transform<U: IntoIterator<Item=f64>> (&self, inputs: U) -> Vec<T> {
        // TODO: make to work on references?
        inputs.into_iter().map(|x| self.transform_single(x)).collect()
    }
}

/// A struct that describes different transformations of the numbers between 0 and 1 to themselves,
/// used for controlling the linearity or nonlinearity of gradients.
#[derive(Debug, PartialEq, Clone)]
pub enum NormalizeMapping {
    /// A normal linear mapping: each number maps to itself.
    Linear,
    /// A cube root mapping: 1/8 would map to 1/2, for example. This has the effect of emphasizing the
    /// differences in the low end of the range, which is useful for some data like sound intensity
    /// that isn't perceived linearly.
    Cbrt,
    /// A generic mapping, taking as a value any function or closure that maps the integers from 0-1
    /// to the same range. This should never fail.
    Generic(fn(f64) -> f64)
}

impl NormalizeMapping {
    /// Performs the given mapping on an input number, with undefined behavior or panics if the given
    /// number is outside of the range (0, 1). Given an input between 0 and 1, should always output
    /// another number in the same range.
    pub fn normalize(&self, x: f64) -> f64 {
        match self {
            &NormalizeMapping::Linear => {
                x
            }
            &NormalizeMapping::Cbrt => {
                x.cbrt()
            }
            &NormalizeMapping::Generic(func) => {
                func(x)
            }
        }
    }
}


/// A gradient colormap: a continuous, evenly-spaced shift between two colors A and B such that 0 maps
/// to A, 1 maps to B, and any number in between maps to a weighted mix of them in a given
/// coordinate space. Uses the gradient functions in the ColorPoint trait to complete this.
/// Out-of-range values are simply clamped to the correct range: calling this on negative numbers
/// will return A, and calling this on numbers larger than 1 will return B.
#[derive(Debug, Clone)]
pub struct GradientColorMap<T: ColorPoint> {
    /// The start of the gradient. Calling this colormap on 0 or any negative number returns this color.
    pub start: T,
    /// The end of the gradient. Calling this colormap on 1 or any larger number returns this color.
    pub end: T,
    /// Any additional added nonlinearity imposed on the gradient: for example, a cube root mapping
    /// emphasizes differences in the low end of the range.
    pub normalization: NormalizeMapping,
    /// Any desired padding: offsets introduced that artificially shift the limits of the
    /// range. Expressed as (new_min, new_max), where both are floats and new_min < new_max. For
    /// example, having padding of (1/8, 1) would remove the lower eighth of the color map while
    /// keeping the overall map smooth and continuous. Padding of (0., 1.) is the default and normal
    /// behavior.
    pub padding: (f64, f64),
}

impl<T: ColorPoint> GradientColorMap<T> {
    /// Constructs a new linear GradientColorMap, without padding, from two colors.
    pub fn new_linear(start: T, end: T) -> GradientColorMap<T> {
        GradientColorMap {
            start,
            end,
            normalization: NormalizeMapping::Linear,
            padding: (0., 1.)
        }
    }
    /// Constructs a new cube root GradientColorMap, without padding, from two colors.
    pub fn new_cbrt(start: T, end: T) -> GradientColorMap<T> {
        GradientColorMap {
            start,
            end,
            normalization: NormalizeMapping::Cbrt,
            padding: (0., 1.)
        }
    }
}

impl<T: ColorPoint> ColorMap<T> for GradientColorMap<T> {
    fn transform_single(&self, x: f64) -> T {
        // clamp between 0 and 1 beforehand
        let clamped = if x < 0. {
            0.
        } else if x > 1. {
            1.
        } else {
            x
        };
        self.start.padded_gradient(&self.end, self.padding.0, self.padding.1) (
            self.normalization.normalize(clamped)
        )
    }
}


/// A colormap that linearly interpolates between a given series of values in an equally-spaced
/// progression. This is modeled off of the `matplotlib` Python library's `ListedColormap`, and is
/// only used to provide reference implementations of the standard matplotlib colormaps. Clamps values
/// outside of 0 to 1.
#[derive(Debug, Clone)]
pub struct ListedColorMap {
    /// The list of values, as a vector of `[f64]` arrays that provide equally-spaced RGB values.
    pub vals: Vec<[f64; 3]>,
}


impl<T: ColorPoint> ColorMap<T> for ListedColorMap {
    /// Linearly interpolates by first finding the two colors on either boundary, and then using a
    /// simple linear gradient. There's no need to instantiate every single Color, because the vast
    /// majority of them aren't important for one computation.
    fn transform_single(&self, x: f64) -> T {
        let clamped = if x < 0. {
            0.
        } else if x > 1. {
            1.
        } else {
            x
        };
        // TODO: keeping every Color in memory might be more efficient for large-scale
        // transformation; if it's a performance issue, try and fix
        
        // now find the two values that bound the clamped x
        // get the index as a floating point: the integers on either side bound it
        // we subtract 1 because 0-n is n+1 numbers, not n
        // otherwise, 1 would map out of range
        let float_ind = clamped * (self.vals.len() as f64 - 1.);
        let ind1 = float_ind.floor() as usize;
        let ind2 = float_ind.ceil() as usize;
        if ind1 == ind2 {
            // x is exactly on the boundary, no interpolation needed
            let arr = self.vals.get(ind1).unwrap(); // guaranteed to be in range
            RGBColor::from(Coord{x: arr[0], y: arr[1], z: arr[2]}).convert()
        } else {
            // interpolate
            let arr1 = self.vals.get(ind1).unwrap();
            let arr2 = self.vals.get(ind2).unwrap();
            let coord1 = Coord{x: arr1[0], y: arr1[1], z: arr1[2]};
            let coord2 = Coord{x: arr2[0], y: arr2[1], z: arr2[2]};
            // now interpolate and convert to the desired type
            let rgb: RGBColor = coord2.weighted_midpoint(&coord1, clamped).into();
            rgb.convert()
        }
    }
}

// now just constructors
impl ListedColorMap {
    // TODO: In the future, I'd like to remove this weird array type bound if possible
    /// Initializes a ListedColorMap from an iterator of arrays [R, G, B].
    pub fn new<T: Iterator<Item=[f64; 3]>> (vals: T) -> ListedColorMap {
        ListedColorMap {
            vals: vals.collect(),
        }
    }
    /// Initializes a viridis colormap, a pleasing blue-green-yellow colormap that is perceptually
    /// uniform with respect to luminance, found in Python's `matplotlib` as the default
    /// colormap.
    pub fn viridis() -> ListedColorMap {
        let vals = matplotlib_cmaps::VIRIDIS_DATA.to_vec();
        ListedColorMap{vals}
    }
    /// Initializes a magma colormap, a pleasing blue-purple-red-yellow map that is perceptually
    /// uniform with respect to luminance, found in Python's `matplotlib.`
    pub fn magma() -> ListedColorMap {
        let vals = matplotlib_cmaps::MAGMA_DATA.to_vec();
        ListedColorMap{vals}
    }
    /// Initializes an inferno colormap, a pleasing blue-purple-red-yellow map similar to magma, but
    /// with a slight shift towards red and yellow, that is perceptually uniform with respect to
    /// luminance, found in Python's `matplotlib.`
    pub fn inferno() -> ListedColorMap {
        let vals = matplotlib_cmaps::INFERNO_DATA.to_vec();
        ListedColorMap{vals}
    }
    /// Initializes a plasma colormap, a pleasing blue-purple-red-yellow map that is perceptually
    /// uniform with respect to luminance, found in Python's `matplotlib.` It eschews the really dark
    /// blue found in inferno and magma, instead starting at a fairly bright blue.
    pub fn plasma() -> ListedColorMap {
        let vals = matplotlib_cmaps::PLASMA_DATA.to_vec();
        ListedColorMap{vals}
    }
}



#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use color::RGBColor;

    #[test]
    fn test_linear_gradient() {
        let red = RGBColor::from_hex_code("#ff0000").unwrap();
        let blue = RGBColor::from_hex_code("#0000ff").unwrap();
        let cmap = GradientColorMap::new_linear(red, blue);
        let vals = vec![-0.2, 0., 1. / 15., 1. / 5., 4. / 5., 1., 100.];
        let cols = cmap.transform(vals);
        let strs = vec!["#FF0000", "#FF0000", "#EE0011", "#CC0033", "#3300CC", "#0000FF", "#0000FF"];
        for (i, col) in cols.into_iter().enumerate() {
            assert_eq!(col.to_string(), strs[i]);
        }
    }
    #[test]
    fn test_cbrt_gradient() {
        let red = RGBColor::from_hex_code("#CC0000").unwrap();
        let blue = RGBColor::from_hex_code("#0000CC").unwrap();
        let cmap = GradientColorMap::new_cbrt(red, blue);
        let vals = vec![-0.2, 0., 1. / 27., 1. / 8., 8. / 27., 1., 100.];
        let cols = cmap.transform(vals);
        let strs = vec!["#CC0000", "#CC0000", "#880044", "#660066", "#440088", "#0000CC", "#0000CC"];
        for (i, col) in cols.into_iter().enumerate() {
            println!("{} {} {}", col.r, col.g, col.b);
            assert_eq!(col.to_string(), strs[i]);
        }
    }
    #[test]
    fn test_padding() {
        let red = RGBColor::from_hex_code("#CC0000").unwrap();
        let blue = RGBColor::from_hex_code("#0000CC").unwrap();
        let mut cmap = GradientColorMap::new_cbrt(red, blue);
        cmap.padding = (0.25, 0.75);
        // essentially, start and end are now #990033 and #330099
        let vals = vec![-0.2, 0., 1. / 27., 1. / 8., 8. / 27., 1., 100.];
        let cols = cmap.transform(vals);
        let strs = vec!["#990033", "#990033", "#770055", "#660066", "#550077", "#330099", "#330099"];
        for (i, col) in cols.into_iter().enumerate() {
            println!("{} {} {}", col.r, col.g, col.b);
            assert_eq!(col.to_string(), strs[i]);
        }
    }
    #[test]
    fn test_mpl_colormaps() {
        let viridis = ListedColorMap::viridis();
        let magma = ListedColorMap::magma();
        let inferno = ListedColorMap::inferno();
        let plasma = ListedColorMap::plasma();
        let vals = vec![-0.2, 0., 0.2, 0.4, 0.6, 0.8, 1., 100.];
        // these values were taken using matplotlib
        let viridis_colors = [
            [ 0.267004,  0.004874,  0.329415],
            [ 0.267004,  0.004874,  0.329415],
            [ 0.253935,  0.265254,  0.529983],
            [ 0.163625,  0.471133,  0.558148],
            [ 0.134692,  0.658636,  0.517649],
            [ 0.477504,  0.821444,  0.318195],
            [ 0.993248,  0.906157,  0.143936],
            [ 0.993248,  0.906157,  0.143936]
        ];
        let magma_colors = [
            [  1.46200000e-03,   4.66000000e-04,   1.38660000e-02],
            [  1.46200000e-03,   4.66000000e-04,   1.38660000e-02],
            [  2.32077000e-01,   5.98890000e-02,   4.37695000e-01],
            [  5.50287000e-01,   1.61158000e-01,   5.05719000e-01],
            [  8.68793000e-01,   2.87728000e-01,   4.09303000e-01],
            [  9.94738000e-01,   6.24350000e-01,   4.27397000e-01],
            [  9.87053000e-01,   9.91438000e-01,   7.49504000e-01],
            [  9.87053000e-01,   9.91438000e-01,   7.49504000e-01]
        ];
        let plasma_colors = [
            [  5.03830000e-02,   2.98030000e-02,   5.27975000e-01],
            [  5.03830000e-02,   2.98030000e-02,   5.27975000e-01],
            [  4.17642000e-01,   5.64000000e-04,   6.58390000e-01],
            [  6.92840000e-01,   1.65141000e-01,   5.64522000e-01],
            [  8.81443000e-01,   3.92529000e-01,   3.83229000e-01],
            [  9.88260000e-01,   6.52325000e-01,   2.11364000e-01],
            [  9.40015000e-01,   9.75158000e-01,   1.31326000e-01],
            [  9.40015000e-01,   9.75158000e-01,   1.31326000e-01]
        ];
        let inferno_colors = [
            [  1.46200000e-03,   4.66000000e-04,   1.38660000e-02],
            [  1.46200000e-03,   4.66000000e-04,   1.38660000e-02],
            [  2.58234000e-01,   3.85710000e-02,   4.06485000e-01],
            [  5.78304000e-01,   1.48039000e-01,   4.04411000e-01],
            [  8.65006000e-01,   3.16822000e-01,   2.26055000e-01],
            [  9.87622000e-01,   6.45320000e-01,   3.98860000e-02],
            [  9.88362000e-01,   9.98364000e-01,   6.44924000e-01],
            [  9.88362000e-01,   9.98364000e-01,   6.44924000e-01]
        ];
        let colors = vec![viridis_colors, magma_colors, inferno_colors, plasma_colors];
        let cmaps = vec![viridis, magma, inferno, plasma];
        for (colors, cmap) in colors.iter().zip(cmaps.iter()) {
            for (ref_arr, test_color) in colors.iter().zip(cmap.transform(vals.clone()).iter()) {
                let ref_color = RGBColor{r: ref_arr[0], g: ref_arr[1], b: ref_arr[2]};
                let deref_test_color: RGBColor = *test_color;
                assert_eq!(deref_test_color.to_string(), ref_color.to_string());
            }
        }
    }
}
