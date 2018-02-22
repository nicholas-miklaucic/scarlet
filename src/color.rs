//! This file defines the `Color` trait, the foundational defining trait of the entire
//! library. Despite the dizzying amount of things `Color` can do in Scarlet, especially with its
//! extending traits, the definition is quite simple: anything that can be converted to and from the
//! [CIE 1931 XYZ space](https://en.wikipedia.org/wiki/CIE_1931_color_space). This color space is
//! common to use as a master space, and Scarlet is no different. What makes XYZ unique is that it
//! can be computed directly from the spectral data of a color. Although Scarlet does not implement
//! this due to its scope, this property makes it possible to derive XYZ colors from real-world data,
//! something that no other color space can do the same way.
//! The thing that makes `XYZColor`, the base implementation of the CIE 1931 XYZ space, special is
//! that it is the only color object in Scarlet that keeps track of its own illuminant data. Every
//! other color space assumes a viewing environment, but because XYZ color maps directly to neural
//! perception it keeps track of what environment the color is being viewed in. This allows Scarlet
//! to translate between color spaces that have different assumptions seamlessly. (If you notice that
//! Scarlet's values for conversions differ from other sources, this may be why: some sources don't
//! do this properly or implement it differently. Scarlet generally follows best practices and
//! industry standards, but file an issue if you feel this is not true.)
//! The essential workflow of `Color`, and therefore Scarlet, is generally like this: convert between
//! different color spaces using the generic [`convert`] method, which allows any `Color` to be
//! interconverted to any other representation. Leverage the specific attributes of each color space
//! if need be (for example, using the hue or luminance attributes), and then convert back to a
//! suitable display space. The many other methods of `Color` make some of the more common such
//! patterns simple to do.

use std::collections::HashMap;
use std::convert::From;
use std::marker::Sized;
use std::num::ParseIntError;
use std::result::Result::Err;
use std::str::FromStr;
use std::string::ToString;

use super::coord::Coord;
use illuminants::Illuminant;
use colors::cielabcolor::CIELABColor;
use colors::cielchcolor::CIELCHColor;
use consts::BRADFORD_TRANSFORM_MAT as BRADFORD;
use consts::STANDARD_RGB_TRANSFORM_MAT as SRGB;
use consts;

use termion::color::{Bg, Fg, Reset, Rgb};
use na::Vector3;

//

/// A point in the CIE 1931 XYZ color space. Although any point in XYZ coordinate space is technically
/// valid, in this library XYZ colors are treated as normalized so that Y=1 is the white point of
/// whatever illuminant is being worked with.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct XYZColor {
    /// The X axis of the CIE 1931 XYZ space, roughly representing the long-wavelength receptors in
    /// the human eye: the red receptors. Usually between 0 and 1, but can range more than that.
    pub x: f64,
    /// The Y axis of the CIE 1931 XYZ space, roughly representing the middle-wavelength receptors in
    /// the human eye. In CIE 1931, this is fudged a little to correspond exactly with perceived
    /// luminance, so while this doesn't exactly map to middle-wavelength receptors it has a far more
    /// useful analogue.
    pub y: f64,
    /// The Z axis of the CIE 1931 XYZ space, roughly representing the short-wavelength receptors in
    /// the human eye. Usually between 0 and 1, but can range more than that.
    pub z: f64,
    /// The illuminant that is assumed to be the lighting environment for this color. Although XYZ
    /// itself describes the human response to a color and so is independent of lighting, it is useful
    /// to consider the question "how would an object in one light look different in another?" and so,
    /// to contain all the information needed to track this, the illuminant is set. See the
    /// [`color_adapt`] method to examine how this is used in the wild.
    pub illuminant: Illuminant,
}

impl XYZColor {
    /// Converts from one illuminant to a different one, such that a human receiving both sets of
    /// sensory stimuli in the corresponding lighting conditions would perceive an object with that
    /// color as not having changed. This process, called [*chromatic
    /// adaptation*](https://en.wikipedia.org/wiki/Chromatic_adaptation), happens subconsciously all
    /// the time: when someone walks into the shade, we don't interpret that shift as their face
    /// turning blue. This process is not at all simple to compute, however, and many different
    /// algorithms for doing so exist: it is most likely that each person has their own idiosyncrasies
    /// with chromatic adaptation and so there is no perfect solution. Scarlet implements the
    /// *Bradford transform*, which is generally acknowledged to be one of the leading chromatic
    /// adaptation transforms. Nonetheless, for exact color science work other models are more
    /// appropriate, such as CIECAM02 if you can measure viewing conditions exactly. This transform
    /// may not give very good results when used with custom illuminants that wildly differ, but with
    /// the standard illuminants it does a very good job.
    /// # Example: The Fabled Dress
    /// The most accessible way of describing color transformation is to take a look at [this
    /// image](https://upload.wikimedia.org/wikipedia/en/a/a8/The_Dress_%28viral_phenomenon%29.png),
    /// otherwise known as "the dress". This showcases in a very apparent fashion the problems with
    /// very ambiguous lighting in chromatic adaptation: the photo is cropped to the point that some
    /// of the population perceives it to be in deep shade and for the dress to therefore be white and
    /// gold, while others perceive instead harsh sunlight and therefore perceive it as black and
    /// blue. (For reference, it is actually black and blue.) Scarlet can help us answer the question
    /// "how would this look to an observer with either judgment about the lighting conditions?"
    /// without needing the human eye!
    /// First, we use a photo editor to pick out two colors that represent both colors of the
    /// dress. Then, we'll change the illuminant directly (without using chromatic adaptation, because
    /// we want to actually change the color), and then we'll adapt back to D65 to represent on a
    /// screen the different colors.
    ///
    /// ```rust
    /// # use scarlet::{Color, RGBColor, Illuminant};
    /// let dress_bg = RGBColor::from_hex_code("#7d6e47").unwrap().to_xyz(Illuminant::D65);
    /// let dress_fg = RGBColor::from_hex_code("#9aabd6").unwrap().to_xyz(Illuminant::D65);
    /// // proposed sunlight illuminant: daylight in North America
    /// // We could exaggerate the effect by creating an illuminant with greater Y value at the white
    /// // point, but this will do
    /// let sunlight = Illuminant::D50;
    /// // proposed "shade" illuminant: created by picking the brightest point on the dress without
    /// glare subjectively, and then treating that as white
    /// let shade_white_point = RGBColor::from_hex_code("#b0c5e4").unwrap().to_xyz(Illuminant::D65);
    /// let shade = Illuminant::Custom([shade_wp.x, shade_wp.y, shade_wp.z]);
    /// // make copies of the colors and set illuminants
    /// let mut black = dress_bg;
    /// let mut blue = dress_fg;
    /// let mut gold = dress_bg;
    /// let mut white = dress_fg;
    /// black.illuminant = sunlight;
    /// blue.illuminant = sunlight;
    /// gold.illuminant = shade;
    /// white.illuminant = shade;
    /// // we can just print them out now: the chromatic adaptation is done automatically to get back
    /// // to the color space of the viewing monitor. This isn't exact, mostly because the shade
    /// // illuminant is entirely fudged, but it's surprisingly good
    /// println!("Black: {} Blue: {}", black.to_string(), blue.to_string());
    /// println!("Gold: {}, White: {}", gold.to_string(), white.to_string());
    /// ```
    pub fn color_adapt(&self, other_illuminant: Illuminant) -> XYZColor {
        // no need to transform if same illuminant
        if other_illuminant == self.illuminant {
            *self
        } else {
            // convert to Bradford RGB space
            let rgb = BRADFORD() * Vector3::new(self.x, self.y, self.z);

            // get the RGB values for the white point of the illuminant we are currently using and
            // the one we want: wr here stands for "white reference", i.e., the one we're converting
            // to
            let rgb_w = BRADFORD() * Vector3::from_column_slice(&self.illuminant.white_point());
            let rgb_wr = BRADFORD() * Vector3::from_column_slice(&other_illuminant.white_point());

            // perform the transform
            // this usually includes a parameter indicating how much you want to adapt, but it's
            // assumed that we want total adaptation: D = 1. Maybe this could change someday?

            // because each white point has already been normalized to Y = 1, we don't need ap
            // factor for it, which simplifies calculation even more than setting D = 1 and makes it
            // just a linear transform
            // scale by the ratio of luminance: it should always be 1, but with rounding error it
            // isn't
            let r_c = rgb[0] * rgb_wr[0] / rgb_w[0];
            let g_c = rgb[1] * rgb_wr[1] / rgb_w[1];
            // there's a slight nonlinearity here that I will omit
            let b_c = rgb[2] * rgb_wr[2] / rgb_w[2];
            // convert back to XYZ using inverse of previous matrix

            let xyz_c = consts::inv(BRADFORD()) * Vector3::new(r_c, g_c, b_c);
            XYZColor {
                x: xyz_c[0],
                y: xyz_c[1],
                z: xyz_c[2],
                illuminant: other_illuminant,
            }
        }
    }
    /// Returns `true` if the given other XYZ color's coordinates are all within acceptable error of
    /// each other, which helps account for necessary floating-point errors in conversions. To test
    /// whether two colors are indistinguishable to humans, use instead
    /// `Color::visually_indistinguishable`.
    /// # Example
    ///
    /// ```
    /// # use scarlet::color::XYZColor;
    /// # use scarlet::illuminants::Illuminant;
    /// let xyz1 = XYZColor{x: 0.3, y: 0., z: 0., illuminant: Illuminant::D65};
    /// // note that the difference in illuminant won't be taken into account
    /// let xyz2 = XYZColor{x: 0.1 + 0.1 + 0.1, y: 0., z: 0., illuminant: Illuminant::D55};
    /// // note that because of rounding error these aren't exactly equal!
    /// assert!(xyz1.x != xyz2.x);
    /// // using approx_equal, we can avoid these sorts of errors
    /// assert!(xyz1.approx_equal(&xyz2));
    pub fn approx_equal(&self, other: &XYZColor) -> bool {
        ((self.x - other.x).abs() <= 1e-15 && (self.y - other.y).abs() <= 1e-15
            && (self.z - other.z).abs() <= 1e-15)
    }

    /// Returns `true` if the given other XYZ color would look identically in a different color
    /// space. Uses an approximate float equality that helps resolve errors due to floating-point
    /// representation, only testing if the two floats are within 0.001 of each other.
    /// # Example
    ///
    /// ```
    /// # use scarlet::color::XYZColor;
    /// # use scarlet::illuminants::Illuminant;
    /// assert!(XYZColor::white_point(Illuminant::D65).approx_visually_equal(&XYZColor::white_point(Illuminant::D50)));
    /// ```
    pub fn approx_visually_equal(&self, other: &XYZColor) -> bool {
        let other_c = other.color_adapt(self.illuminant);
        self.approx_equal(&other_c)
    }
    /// Gets the XYZColor corresponding to pure white in the given light environment.
    /// # Example
    ///
    /// ```
    /// # use scarlet::color::XYZColor;
    /// # use scarlet::illuminants::Illuminant;
    /// let white1 = XYZColor::white_point(Illuminant::D65);
    /// let white2 = XYZColor::white_point(Illuminant::D50);
    /// assert!(white1.approx_visually_equal(&white2));
    /// ```
    pub fn white_point(illuminant: Illuminant) -> XYZColor {
        let wp = illuminant.white_point();
        XYZColor {
            x: wp[0],
            y: wp[1],
            z: wp[2],
            illuminant,
        }
    }
}

/// A trait that includes any color representation that can be converted to and from the CIE 1931 XYZ
/// color space.
pub trait Color: Sized {
    /// Converts from a color in CIE 1931 XYZ to the given color type.
    fn from_xyz(XYZColor) -> Self;
    /// Converts from the given color type to a color in CIE 1931 XYZ space. Because most color types
    /// don't include illuminant information, it is provided instead, as an enum. For most
    /// applications, D50 or D65 is a good choice.
    fn to_xyz(&self, illuminant: Illuminant) -> XYZColor;
    /// Converts the given Color to a different Color type, without consuming the curreppnt color. `T`
    /// is the color that is being converted to.  This currently converts back and forth using the
    /// D50 standard illuminant. However, this shouldn't change the actual value if the color
    /// conversion methods operate correctly, and this value should not be relied upon and can be
    /// changed without notice.
    fn convert<T: Color>(&self) -> T {
        // theoretically, the illuminant shouldn't matter as long as the color conversions are
        // correct. D50 is a common gamut for use in internal conversions, so for spaces like CIELAB
        // it will produce the least error
        T::from_xyz(self.to_xyz(Illuminant::D50))
    }
    /// "Colors" a given piece of text with terminal escape codes to allow it to be printed out in the
    /// given foreground color. Will cause problems with terminals that do not support truecolor.
    fn write_colored_str(&self, text: &str) -> String {
        let rgb: RGBColor = self.convert();
        rgb.base_write_colored_str(text)
    }
    /// Returns a string which, when printed in a truecolor-supporting terminal, will hopefully have
    /// both the foreground and background of the desired color, appearing as a complete square.
    fn write_color(&self) -> String {
        let rgb: RGBColor = self.convert();
        rgb.base_write_color()
    }

    /// Gets the generally most accurate version of hue for a given color: the hue coordinate in
    /// CIELCH. There are generally considered four "unique hues" that humans perceive as not
    /// decomposable into other hues (when mixing additively): these are red, yellow, green, and
    /// blue. These unique hues have values of 0, 90, 180, and 270 degrees respectively, with other
    /// colors interpolated between them. This returned value will never be outside the range 0 to
    /// 360.
    fn hue(&self) -> f64 {
        let lch: CIELCHColor = self.convert();
        lch.h
    }

    /// Sets a perceptually-accurate version hue of a color, even if the space itself does not have a
    /// conception of hue. This uses the CIELCH version of hue. To use another one, simply convert and
    /// set it manually. If the given hue is not between 0 and 360, it is shifted in that range by
    /// adding multiples of 360.
    fn set_hue(&mut self, new_hue: f64) -> () {
        let mut lch: CIELCHColor = self.convert();
        lch.h = if new_hue >= 0.0 && new_hue <= 360.0 {
            new_hue
        } else if new_hue < 0.0 {
            new_hue - 360.0 * (new_hue / 360.0).floor()
        } else {
            new_hue - 360.0 * (new_hue / 360.0).ceil()
        };
        *self = lch.convert();
    }

    /// Gets a perceptually-accurate version of lightness as a value from 0 to 100, where 0 is black
    /// and 100 is pure white. The exact value used is CIELAB's definition of luminance, which is
    /// generally considered a very good standard. Note that this is nonlinear with respect to the
    /// physical amount of light emitted: a material with 18% reflectance has a lightness value of 50,
    /// not 18.
    fn lightness(&self) -> f64 {
        let lab: CIELABColor = self.convert();
        lab.l
    }

    /// Sets a perceptually-accurate version of lightness, which ranges between 0 and 100 for visible
    /// colors. Any values outside of this range will be clamped within it.
    fn set_lightness(&mut self, new_lightness: f64) -> () {
        let mut lab: CIELABColor = self.convert();
        lab.l = if new_lightness >= 0.0 && new_lightness <= 100.0 {
            new_lightness
        } else if new_lightness < 0.0 {
            0.0
        } else {
            100.0
        };
        *self = lab.convert()
    }

    /// Gets a perceptually-accurate version of *chroma*, defined as colorfulness relative to a
    /// similarly illuminated white. This has no explicit upper bound, but is always positive and
    /// generally between 0 and 180 for visible colors. This is done using the CIELCH model.
    fn chroma(&self) -> f64 {
        let lch: CIELCHColor = self.convert();
        lch.c
    }

    /// Sets a perceptually-accurate version of *chroma*, defined as colorfulness relative to a
    /// similarly illuminated white. Uses CIELCH's defintion of chroma for implementation. Any value
    /// below 0 will be clamped up to 0, but because the upper bound depends on the hue and
    /// lightness no clamping will be done. This means that this method has a higher chance than
    /// normal of producing imaginary colors and any output from this method should be checked.
    fn set_chroma(&mut self, new_chroma: f64) -> () {
        let mut lch: CIELCHColor = self.convert();
        lch.c = if new_chroma < 0.0 { 0.0 } else { new_chroma };
        *self = lch.convert();
    }

    /// Gets a perceptually-accurate version of *saturation*, defined as chroma relative to
    /// lightness. Generally ranges from 0 to around 10, although exact bounds are tricky. from This
    /// means that e.g., a very dark purple could be very highly saturated even if it does not seem
    /// so relative to lighter colors. This is computed using the CIELCH model and computing chroma
    /// divided by lightness: if the lightness is 0, the saturation is also said to be 0. There is
    /// no official formula except ones that require more information than this model of colors has,
    /// but the CIELCH formula is fairly standard.
    fn saturation(&self) -> f64 {
        let lch: CIELCHColor = self.convert();
        if lch.l == 0.0 {
            0.0
        } else {
            lch.c / lch.l
        }
    }

    /// Sets a perceptually-accurate version of *saturation*, defined as chroma relative to
    /// lightness. Any negative value will be clamped to 0, but because the maximum saturation is not
    /// well-defined any positive value will be used as is: this means that this method is more likely
    /// than others to produce imaginary colors. Uses the CIELCH color space.
    fn set_saturation(&mut self, new_sat: f64) -> () {
        let mut lch: CIELCHColor = self.convert();
        lch.c = if new_sat < 0.0 {
            0.0
        } else {
            new_sat * lch.l
        };
        *self = lch.convert();
    }

    /// Returns a new Color of the same type as before, but with chromaticity removed: effectively,
    /// a color created solely using a mix of black and white that has the same lightness as
    /// before. This uses the CIELAB luminance definition, which is considered a good standard and is
    /// perceptually accurate for the most part.
    fn grayscale(&self) -> Self
    where
        Self: Sized,
    {
        let mut lch: CIELCHColor = self.convert();
        lch.c = 0.0;
        lch.convert()
    }

    /// Returns a metric of the distance between the given color and another that attempts to
    /// accurately reflect human perception. This is done by using the CIEDE2000 difference formula,
    /// the current international and industry standard. The result, being a distance, will never be
    /// negative: it has no defined upper bound, although anything larger than 100 would be very
    /// extreme. A distance of 1.0 is conservatively the smallest possible noticeable difference:
    /// anything that is below 1.0 is almost guaranteed to be indistinguishable to most people.
    fn distance<T: Color>(&self, other: &T) -> f64 {
        // implementation reference found here:
        // https://pdfs.semanticscholar.org/969b/c38ea067dd22a47a44bcb59c23807037c8d8.pdf

        // I'm going to match the notation in that text pretty much exactly: it's the only way to
        // keep this both concise and readable

        // first convert to LAB
        let lab1: CIELABColor = self.convert();
        let lab2: CIELABColor = other.convert();
        // step 1: calculation of C and h
        // the method hypot returns sqrt(a^2 + b^2)
        let c_star_1: f64 = lab1.a.hypot(lab1.b);
        let c_star_2: f64 = lab2.a.hypot(lab2.b);

        let c_bar_ab: f64 = (c_star_1 + c_star_2) / 2.0;
        let g = 0.5 * (1.0 - ((c_bar_ab.powi(7)) / (c_bar_ab.powi(7) + 25.0f64.powi(7))).sqrt());

        let a_prime_1 = (1.0 + g) * lab1.a;
        let a_prime_2 = (1.0 + g) * lab2.a;

        let c_prime_1 = a_prime_1.hypot(lab1.b);
        let c_prime_2 = a_prime_2.hypot(lab2.b);

        // this closure simply does the atan2 like CIELCH, but safely accounts for a == b == 0
        // we're gonna do this twice, so I just use a closure
        let h_func = |a: f64, b: f64| {
            if a == 0.0 && b == 0.0 {
                0.0
            } else {
                let val = b.atan2(a).to_degrees();
                if val < 0.0 {
                    val + 360.0
                } else {
                    val
                }
            }
        };

        let h_prime_1 = h_func(a_prime_1, lab1.b);
        let h_prime_2 = h_func(a_prime_2, lab2.b);

        // step 2: computing delta L, delta C, and delta H
        // take a deep breath, you got this!

        let delta_l = lab2.l - lab1.l;
        let delta_c = c_prime_2 - c_prime_1;
        // essentially, compute the difference in hue but keep it in the right range
        let delta_angle_h = if c_prime_1 * c_prime_2 == 0.0 {
            0.0
        } else if (h_prime_2 - h_prime_1).abs() <= 180.0 {
            h_prime_2 - h_prime_1
        } else if h_prime_2 - h_prime_1 > 180.0 {
            h_prime_2 - h_prime_1 - 360.0
        } else {
            h_prime_2 - h_prime_1 + 360.0
        };
        // now get the Cartesian equivalent of the angle difference in hue
        // this also corrects for chromaticity mattering less at low luminances
        let delta_h =
            2.0 * (c_prime_1 * c_prime_2).sqrt() * (delta_angle_h / 2.0).to_radians().sin();

        // step 3: the color difference
        // if you're reading this, it's not too late to back out
        let l_bar_prime = (lab1.l + lab2.l) / 2.0;
        let c_bar_prime = (c_prime_1 + c_prime_2) / 2.0;
        let h_bar_prime = if c_prime_1 * c_prime_2 == 0.0 {
            h_prime_1 + h_prime_2
        } else if (h_prime_2 - h_prime_1).abs() <= 180.0 {
            (h_prime_1 + h_prime_2) / 2.0
        } else {
            if h_prime_1 + h_prime_2 < 360.0 {
                (h_prime_1 + h_prime_2 + 360.0) / 2.0
            } else {
                (h_prime_1 + h_prime_2 - 360.0) / 2.0
            }
        };

        // we're gonna use this a lot
        let deg_cos = |x: f64| x.to_radians().cos();

        let t = 1.0 - 0.17 * deg_cos(h_bar_prime - 30.0) + 0.24 * deg_cos(2.0 * h_bar_prime)
            + 0.32 * deg_cos(3.0 * h_bar_prime + 6.0)
            - 0.20 * deg_cos(4.0 * h_bar_prime - 63.0);

        let delta_theta = 30.0 * (-(((h_bar_prime - 275.0) / 25.0)).powi(2)).exp();
        let r_c = 2.0 * (c_bar_prime.powi(7) / (c_bar_prime.powi(7) + 25.0f64.powi(7))).sqrt();
        let s_l = 1.0
            + ((0.015 * (l_bar_prime - 50.0).powi(2))
                / (20.0 + (l_bar_prime - 50.0).powi(2)).sqrt());
        let s_c = 1.0 + 0.045 * c_bar_prime;
        let s_h = 1.0 + 0.015 * c_bar_prime * t;
        let r_t = -r_c * (2.0 * delta_theta).to_radians().sin();
        // finally, the end result
        // in the original there are three parametric weights, used for weighting differences in
        // lightness, chroma, or hue. In pretty much any application, including this one, all of
        // these are 1, so they're omitted
        ((delta_l / s_l).powi(2) + (delta_c / s_c).powi(2) + (delta_h / s_h).powi(2)
            + r_t * (delta_c / s_c) * (delta_h / s_h))
            .sqrt()
    }
    /// Using the metric that two colors with a CIEDE2000 distance of less than 1 are
    /// indistinguishable, determines whether two colors are visually distinguishable from each
    /// other.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scarlet::color::{RGBColor, Color};
    ///
    /// let color1 = RGBColor::from_hex_code("#123456").unwrap();
    /// let color2 = RGBColor::from_hex_code("#123556").unwrap();
    /// let color3 = RGBColor::from_hex_code("#333333").unwrap();
    ///
    /// assert!(color1.visually_indistinguishable(&color2)); // yes, they are visually indistinguishable
    /// assert!(color2.visually_indistinguishable(&color1)); // yes, the same two points
    /// assert!(!color1.visually_indistinguishable(&color3)); // not visually distinguishable
    /// ```
    fn visually_indistinguishable<T: Color>(&self, other: &T) -> bool {
        self.distance(other) <= 1.0
    }
}

impl Color for XYZColor {
    fn from_xyz(xyz: XYZColor) -> XYZColor {
        xyz
    }
    #[allow(unused_variables)]
    fn to_xyz(&self, illuminant: Illuminant) -> XYZColor {
        *self
    }
}

#[derive(Debug, Copy, Clone)]
/// A color with red, green, and blue primaries of specified intensity, specifically in the sRGB
/// gamut: most computer screens use this to display colors. The attributes `r`, `g`, and `b` are
/// floating-point numbers from 0 to 1 for visible colors, allowing the avoidance of rounding errors
/// or clamping errors when converting to and from RGB. Many conveniences are afforded so that working
/// with RGB as if it were instead three integers from 0-255 is painless. Note that the integers
/// generated from the underlying floating-point numbers round away from 0.
pub struct RGBColor {
    // The red component. Ranges from 0 to 1 for numbers displayable by sRGB machines.
    pub r: f64,
    // The green component. Ranges from 0 to 1 for numbers displayable by sRGB machines.
    pub g: f64,
    // The blue component. Ranges from 0 to 1 for numbers displayable by sRGB machines.
    pub b: f64,
    // TODO: add exact unclamped versions of each of these
}

impl RGBColor {
    /// Gets an 8-byte version of the red component, as a `u8`. Clamps values outside of the range 0-1
    /// and discretizes, so this may not correspond to the exact values kept internally.
    pub fn int_r(&self) -> u8 {
        // first clamp, then multiply by 255, round, and discretize
        if self.r < 0.0 {
            0_u8
        } else if self.r > 1.0 {
            255_u8
        } else {
            (self.r * 255.0).round() as u8
        }
    }
    /// Gets an 8-byte version of the green component, as a `u8`. Clamps values outside of the range 0-1
    /// and discretizes, so this may not correspond to the exact values kept internally.
    pub fn int_g(&self) -> u8 {
        // first clamp, then multiply by 255, round, and discretize
        if self.g < 0.0 {
            0_u8
        } else if self.g > 1.0 {
            255_u8
        } else {
            (self.g * 255.0).round() as u8
        }
    }
    /// Gets an 8-byte version of the blue component, as a `u8`. Clamps values outside of the range 0-1
    /// and discretizes, so this may not correspond to the exact values kept internally.
    pub fn int_b(&self) -> u8 {
        // first clamp, then multiply by 255, round, and discretize
        if self.b < 0.0 {
            0_u8
        } else if self.b > 1.0 {
            255_u8
        } else {
            (self.b * 255.0).round() as u8
        }
    }
    /// Purely for convenience: gives a tuple with the three integer versions of the components. Used
    /// over standard conversion traits to avoid ambiguous operations.
    pub fn int_rgb_tup(&self) -> (u8, u8, u8) {
        (self.int_r(), self.int_g(), self.int_b())
    }

    /// Purely for convenience: gives a slice with the three integer versions of the components. Used
    /// over standard conversion traits to avoid ambiguous operations.
    pub fn int_rgb(&self) -> [u8; 3] {
        [self.int_r(), self.int_g(), self.int_b()]
    }

    /// Given a string, returns that string wrapped in codes that will color the foreground. Used for
    /// the trait implementation of write_colored_str, which should be used instead.
    pub fn base_write_colored_str(&self, text: &str) -> String {
        format!(
            "{code}{text}{reset}",
            code = Fg(Rgb(self.int_r(), self.int_g(), self.int_b())),
            text = text,
            reset = Fg(Reset)
        )
    }
    pub fn base_write_color(&self) -> String {
        format!(
            "{bg}{fg}{text}{reset_fg}{reset_bg}",
            bg = Bg(Rgb(self.int_r(), self.int_g(), self.int_b())),
            fg = Fg(Rgb(self.int_r(), self.int_g(), self.int_b())),
            text = "■",
            reset_fg = Fg(Reset),
            reset_bg = Bg(Reset),
        )
    }
}

impl PartialEq for RGBColor {
    fn eq(&self, other: &RGBColor) -> bool {
        self.r == other.r && self.g == other.g && self.b == other.b
    }
}

impl From<(u8, u8, u8)> for RGBColor {
    fn from(rgb: (u8, u8, u8)) -> RGBColor {
        let (r, g, b) = rgb;
        RGBColor {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
        }
    }
}

impl Into<(u8, u8, u8)> for RGBColor {
    fn into(self) -> (u8, u8, u8) {
        (self.int_r(), self.int_g(), self.int_b())
    }
}

impl From<Coord> for RGBColor {
    fn from(c: Coord) -> RGBColor {
        RGBColor {
            r: c.x,
            g: c.y,
            b: c.z,
        }
    }
}

impl Into<Coord> for RGBColor {
    fn into(self) -> Coord {
        Coord {
            x: self.r,
            y: self.g,
            z: self.b,
        }
    }
}

impl ToString for RGBColor {
    fn to_string(&self) -> String {
        format!(
            "#{:02X}{:02X}{:02X}",
            self.int_r(),
            self.int_g(),
            self.int_b()
        )
    }
}

impl Color for RGBColor {
    fn from_xyz(xyz: XYZColor) -> RGBColor {
        // sRGB uses D65 as the assumed illuminant: convert the given value to that
        let xyz_d65 = xyz.color_adapt(Illuminant::D65);
        // first, get linear RGB values (i.e., without gamma correction)
        // https://en.wikipedia.org/wiki/SRGB#Specification_of_the_transformation

        let lin_rgb_vec = SRGB() * Vector3::new(xyz_d65.x, xyz_d65.y, xyz_d65.z);
        // now we scale for gamma correction
        let gamma_correct = |x: &f64| {
            if x <= &0.0031308 {
                &12.92 * x
            } else {
                &1.055 * x.powf(&1.0 / &2.4) - &0.055
            }
        };
        let float_vec: Vec<f64> = lin_rgb_vec.iter().map(gamma_correct).collect();
        RGBColor {
            r: float_vec[0],
            g: float_vec[1],
            b: float_vec[2],
        }
    }

    fn to_xyz(&self, illuminant: Illuminant) -> XYZColor {
        let uncorrect_gamma = |x: &f64| {
            if x <= &0.04045 {
                x / &12.92
            } else {
                ((x + &0.055) / &1.055).powf(2.4)
            }
        };
        let rgb_vec = Vector3::from_iterator([self.r, self.g, self.b].iter().map(uncorrect_gamma));

        // invert the matrix multiplication used in from_xyz()
        let xyz_vec = consts::inv(SRGB()) * rgb_vec;

        // sRGB, which this is based on, uses D65 as white, but you can convert to whatever
        // illuminant is specified
        let converted = XYZColor {
            x: xyz_vec[0],
            y: xyz_vec[1],
            z: xyz_vec[2],
            illuminant: Illuminant::D65,
        };
        converted.color_adapt(illuminant)
    }
}

/// An error type that results from an invalid attempt to convert a string into an RGB color.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum RGBParseError {
    /// This indicates that function syntax was acceptable, but the numbers were out of range, such as
    /// the invalid string `"rgb(554, 23, 553)"`.
    OutOfRange,
    /// This indicates that the hex string was malformed in some way.
    InvalidHexSyntax,
    /// This indicates a syntax error in the string that was supposed to be a valid rgb( function.
    InvalidFuncSyntax,
    /// This indicated an invalid color name was supplied to the `from_color_name()` function.
    InvalidX11Name,
}

impl From<ParseIntError> for RGBParseError {
    fn from(_err: ParseIntError) -> RGBParseError {
        RGBParseError::OutOfRange
    }
}

impl RGBColor {
    /// Given a string that represents a hex code, returns the RGB color that the given hex code
    /// represents. Four formats are accepted: `"#rgb"` as a shorthand for `"#rrggbb"`, `#rrggbb` by
    /// itself, and either of those formats without `#`: `"rgb"` or `"rrggbb"` are acceptable. Returns
    /// a ColorParseError if the given string does not follow one of these formats.
    pub fn from_hex_code(hex: &str) -> Result<RGBColor, RGBParseError> {
        let mut chars: Vec<char> = hex.chars().collect();
        // check if leading hex, remove if so
        if chars[0] == '#' {
            chars.remove(0);
        }
        // can only have 3 or 6 characters: error if not so
        if chars.len() != 3 && chars.len() != 6 {
            Err(RGBParseError::InvalidHexSyntax)
        // now split on invalid hex
        } else if !chars.iter().all(|&c| "0123456789ABCDEFabcdef".contains(c)) {
            Err(RGBParseError::InvalidHexSyntax)
        } else {
            // split on whether it's #rgb or #rrggbb
            if chars.len() == 6 {
                let mut rgb: Vec<u8> = Vec::new();
                for _i in 0..3 {
                    // this should never fail, logically, but if by some miracle it did it'd just
                    // return an OutOfRangeError
                    rgb.push(
                        u8::from_str_radix(chars.drain(..2).collect::<String>().as_str(), 16)
                            .unwrap(),
                    );
                }
                Ok(RGBColor::from((rgb[0], rgb[1], rgb[2])))
            } else {
                // len must be 3 from earlier
                let mut rgb: Vec<u8> = Vec::new();
                for _i in 0..3 {
                    // again, this shouldn't ever fail, but if it did it'd just return an
                    // OutOfRangeError
                    let c: Vec<char> = chars.drain(..1).collect();
                    rgb.push(
                        u8::from_str_radix(
                            c.iter().chain(c.iter()).collect::<String>().as_str(),
                            16,
                        ).unwrap(),
                    );
                }
                Ok(RGBColor::from((rgb[0], rgb[1], rgb[2])))
            }
        }
    }
    /// Gets the RGB color corresponding to an X11 color name. Case is ignored.
    pub fn from_color_name(name: &str) -> Result<RGBColor, RGBParseError> {
        // this is the full list of X11 color names
        // I used a Python script to process it from this site:
        // https://github.com/bahamas10/css-color-names/blob/master/css-color-names.json let
        // I added the special "transparent" referring to #00000000
        let color_names: Vec<&str> = consts::X11_NAMES.to_vec();
        let color_codes: Vec<&str> = consts::X11_COLOR_CODES.to_vec();
        let mut names_to_codes = HashMap::new();

        for (i, color_name) in color_names.iter().enumerate() {
            names_to_codes.insert(color_name, color_codes[i]);
        }

        // now just return the converted value or raise one if not in hashmap
        match names_to_codes.get(&name.to_lowercase().as_str()) {
            None => Err(RGBParseError::InvalidX11Name),
            Some(x) => Self::from_hex_code(x),
        }
    }
}

impl FromStr for RGBColor {
    type Err = RGBParseError;

    fn from_str(s: &str) -> Result<RGBColor, RGBParseError> {
        match RGBColor::from_hex_code(s) {
            Err(_e) => RGBColor::from_color_name(s),
            Ok(rgb) => Ok(rgb)
        }
    }
}


#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;    

    #[test]
    fn test_visual_distinguishability(){
        let color1 = RGBColor::from_hex_code("#123456").unwrap();
        let color2 = RGBColor::from_hex_code("#123556").unwrap();
        let color3 = RGBColor::from_hex_code("#333333").unwrap();
        assert!(color1.visually_indistinguishable(&color2));
        assert!(color2.visually_indistinguishable(&color1));
        assert!(!color1.visually_indistinguishable(&color3));
    }

    #[test]
    fn can_display_colors() {
        let range = 120;
        let mut col;
        let mut line;
        let mut c;
        let mut h;
        println!("");
        for i in 0..range {
            h = (i as f64) / (range as f64) * 360.;
            line = String::new();
            for j in 0..range {
                c = j as f64;
                col = CIELCHColor{l: 70., c: c / 2., h};
                line += col.write_color().as_str();
            }
            println!("{}", line);
        }
        println!("");
    }

    #[test]
    fn xyz_to_rgb() {
        let xyz = XYZColor {
            x: 0.41874,
            y: 0.21967,
            z: 0.05649,
            illuminant: Illuminant::D65,
        };
        let rgb: RGBColor = xyz.convert();
        assert_eq!(rgb.int_r(), 254);
        assert_eq!(rgb.int_g(), 23);
        assert_eq!(rgb.int_b(), 55);
    }

    #[test]
    fn rgb_to_xyz() {
        let rgb = RGBColor::from((45, 28, 156));
        let xyz: XYZColor = rgb.to_xyz(Illuminant::D65);
        // these won't match exactly cuz floats, so I just check within a margin
        assert!((xyz.x - 0.0750).abs() <= 0.01);
        assert!((xyz.y - 0.0379).abs() <= 0.01);
        assert!((xyz.z - 0.3178).abs() <= 0.01);
    }
    #[test]
    fn test_rgb_to_string() {
        let c1 = RGBColor::from((0, 0, 0));
        let c2 = RGBColor::from((244, 182, 33));
        let c3 = RGBColor::from((0, 255, 0));
        assert_eq!(c1.to_string(), "#000000");
        assert_eq!(c2.to_string(), "#F4B621");
        assert_eq!(c3.to_string(), "#00FF00");
    }
    #[test]
    fn test_xyz_color_adaptation() {
        // I can literally not find a single API or something that does this so I can check the
        // values, so I'll just hope that it's good enough to check that converting between several
        // illuminants and back again gets something good
        let c1 = XYZColor {
            x: 0.5,
            y: 0.75,
            z: 0.6,
            illuminant: Illuminant::D65,
        };
        let c2 = c1.color_adapt(Illuminant::D50).color_adapt(Illuminant::D55);
        let c3 = c1.color_adapt(Illuminant::D75).color_adapt(Illuminant::D55);
        println!("{} {} {}", c1.x, c1.y, c1.z);
        println!("{} {} {}", c2.x, c2.y, c2.z);
        println!("{} {} {}", c3.x, c3.y, c3.z);
        assert!((c3.x - c2.x).abs() <= 0.01);
        assert!((c3.y - c2.y).abs() <= 0.01);
        assert!((c3.z - c2.z).abs() <= 0.01);
    }
    #[test]
    fn test_error_buildup_color_adaptation() {
        // this is essentially just seeing how consistent the inverse function is for the Bradford
        // transform
        let xyz = XYZColor {
            x: 0.5,
            y: 0.4,
            z: 0.6,
            illuminant: Illuminant::D65,
        };
        let mut xyz2;
        const MAX_ITERS_UNTIL_UNACCEPTABLE_ERROR: usize = 100;
        for i in 0..MAX_ITERS_UNTIL_UNACCEPTABLE_ERROR {
            let lum = [
                Illuminant::D50,
                Illuminant::D55,
                Illuminant::D65,
                Illuminant::D75,
            ][i % 4];
            xyz2 = xyz.color_adapt(lum);
            assert!(xyz2.approx_visually_equal(&xyz));
        }
    }
    #[test]
    fn test_chromatic_adapation_to_same_light() {
        let xyz = XYZColor {
            x: 0.4,
            y: 0.6,
            z: 0.2,
            illuminant: Illuminant::D65,
        };
        let xyz2 = xyz.color_adapt(Illuminant::D65);
        assert_eq!(xyz, xyz2);
    }
    #[test]
    fn fun_dress_color_adaptation_demo() {
        // the famous dress colors, taken completely out of the lighting conditions using GIMP
        let dress_bg = RGBColor::from_hex_code("#7d6e47").unwrap().to_xyz(Illuminant::D65); let
        dress_fg = RGBColor::from_hex_code("#9aabd6").unwrap().to_xyz(Illuminant::D65);

        // helper closure to print block of color
        let block_size = 50;
        let print_col = |c: XYZColor| {
            println!();
            for _i in 0..block_size {
                println!("{}", c.write_color().repeat(block_size));
            }
        };

        // make two "proposed" illuminants: different observers disagree on which one from the image!
        // bright sunlight, clearly the incorrect one (actually, correct, just the one I don't see)
        let sunlight = Illuminant::D50;  // essentially daylight in East US, approximately
        // dark shade, clear the correct one (actually, incorrect, but the one I see)
        // to get this, I used GIMP again and picked the brightest point on the dress
        let dress_wp = RGBColor::from_hex_code("#b0c5e4").unwrap();
        let shade_wp = dress_wp.to_xyz(Illuminant::D65);
        let shade = Illuminant::Custom([shade_wp.x, shade_wp.y, shade_wp.z]);
        // print alternate blocks of color: first the dress interpreted in sunlight (black and blue),
        // then the dress interpreted in shade (white and gold)
        let mut black = dress_bg;
        let mut blue = dress_fg;
        black.illuminant = sunlight;
        blue.illuminant = sunlight;

        let mut gold = dress_bg;
        let mut white = dress_fg;
        gold.illuminant = shade;
        white.illuminant = shade;
        
        print_col(black);
        print_col(blue);
        print_col(gold);
        print_col(white);
    }

    #[test]
    #[ignore]
    fn fun_color_adaptation_demo() {
        println!();
        let w: usize = 120;
        let h: usize = 60;
        let d50_wp = Illuminant::D50.white_point();
        let d75_wp = Illuminant::D75.white_point();
        let d50 = XYZColor {
            x: d50_wp[0],
            y: d50_wp[1],
            z: d50_wp[2],
            illuminant: Illuminant::D65,
        };
        let d75 = XYZColor {
            x: d75_wp[0],
            y: d75_wp[1],
            z: d75_wp[2],
            illuminant: Illuminant::D65,
        };
        for _ in 0..h + 1 {
            println!(
                "{}{}",
                d50.write_color().repeat(w / 2),
                d75.write_color().repeat(w / 2)
            );
        }

        println!();
        println!();
        let y = 0.5;
        println!();
        for i in 0..(h + 1) {
            let mut line = String::from("");
            let x = i as f64 * 0.9 / h as f64;
            for j in 0..(w / 2) {
                let z = j as f64 * 0.9 / w as f64;
                line.push_str(
                    XYZColor {
                        x,
                        y,
                        z,
                        illuminant: Illuminant::D50,
                    }.write_color()
                        .as_str(),
                );
            }
            for j in (w / 2)..(w + 1) {
                let z = j as f64 * 0.9 / w as f64;
                line.push_str(
                    XYZColor {
                        x,
                        y,
                        z,
                        illuminant: Illuminant::D75,
                    }.write_color()
                        .as_str(),
                );
            }
            println!("{}", line);
        }
        println!();
        println!();
        for i in 0..(h + 1) {
            let mut line = String::from("");
            let x = i as f64 * 0.9 / h as f64;
            for j in 0..w {
                let z = j as f64 * 0.9 / w as f64;
                line.push_str(
                    XYZColor {
                        x,
                        y,
                        z,
                        illuminant: Illuminant::D65,
                    }.write_color()
                        .as_str(),
                );
            }
            println!("{}", line);
        }
    }
    #[test]
    fn test_rgb_from_hex() {
        // test rgb format
        let rgb = RGBColor::from_hex_code("#172844").unwrap();
        assert_eq!(rgb.int_r(), 23);
        assert_eq!(rgb.int_g(), 40);
        assert_eq!(rgb.int_b(), 68);
        // test with letters and no hex
        let rgb = RGBColor::from_hex_code("a1F1dB").unwrap();
        assert_eq!(rgb.int_r(), 161);
        assert_eq!(rgb.int_g(), 241);
        assert_eq!(rgb.int_b(), 219);
        // test for error if 7 chars
        let rgb = RGBColor::from_hex_code("#1244444");
        assert!(match rgb {
            Err(x) if x == RGBParseError::InvalidHexSyntax => true,
            _ => false,
        });
        // test for error if invalid hex chars
        let rgb = RGBColor::from_hex_code("#ffggbb");
        assert!(match rgb {
            Err(x) if x == RGBParseError::InvalidHexSyntax => true,
            _ => false,
        });
    }
    #[test]
    fn test_rgb_from_name() {
        let rgb = RGBColor::from_color_name("yeLlowgreEn").unwrap();
        assert_eq!(rgb.int_r(), 154);
        assert_eq!(rgb.int_g(), 205);
        assert_eq!(rgb.int_b(), 50);
        // test error
        let rgb = RGBColor::from_color_name("thisisnotavalidnamelol");
        assert!(match rgb {
            Err(x) if x == RGBParseError::InvalidX11Name => true,
            _ => false,
        });
    }
    #[test]
    fn test_to_string() {
        for hex in ["#000000", "#ABCDEF", "#1A2B3C", "#D00A12", "#40AA50"].iter() {
            assert_eq!(*hex, RGBColor::from_hex_code(hex).unwrap().to_string());
        }
    }
    #[test]
    fn test_ciede2000() {
        // this implements the fancy test cases found here:
        // https://pdfs.semanticscholar.org/969b/c38ea067dd22a47a44bcb59c23807037c8d8.pdf
        let l_1 = vec![
            50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0,
            50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 60.2574, 63.0109, 61.2901,
            35.0831, 22.7233, 36.4612, 90.8027, 90.9257, 6.7747, 2.0776,
        ];
        let l_2 = vec![
            50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0,
            50.0, 50.0, 73.0, 61.0, 56.0, 58.0, 50.0, 50.0, 50.0, 50.0, 60.4626, 62.8187, 61.4292,
            35.0232, 23.0331, 36.2715, 91.1528, 88.6381, 5.8714, 0.9033,
        ];
        let a_1 = vec![
            2.6772, 3.1571, 2.8361, -1.3802, -1.1848, -0.9009, 0.0, -1.0, 2.49, 2.49, 2.49, 2.49,
            -0.001, -0.001, -0.001, 2.5, 2.5, 2.5, 2.5, 2.5, 2.5, 2.5, 2.5, 2.5, -34.0099,
            -31.0961, 3.7196, -44.1164, 20.0904, 47.858, -2.0831, -0.5406, -0.2908, 0.0795,
        ];
        let a_2 = vec![
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, -2.49, -2.49, -2.49, -2.49, 0.0009, 0.001,
            0.0011, 0.0, 25.0, -5.0, -27.0, 24.0, 3.1736, 3.2972, 1.8634, 3.2592, -34.1751,
            -29.7946, 2.248, -40.0716, 14.973, 50.5065, -1.6435, -0.8985, -0.0985, -0.0636,
        ];
        let b_1 = vec![
            -79.7751, -77.2803, -74.02, -84.2814, -84.8006, -85.5211, 0.0, 2.0, -0.001, -0.001,
            -0.001, -0.001, 2.49, 2.49, 2.49, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 36.2677,
            -5.8663, -5.3901, 3.7933, -46.6940, 18.3852, 1.441, -0.9208, -2.4247, -1.135,
        ];
        let b_2 = vec![
            -82.7485, -82.7485, -82.7485, -82.7485, -82.7485, -82.7485, 2.0, 0.0, 0.0009, 0.001,
            0.0011, 0.0012, -2.49, -2.49, -2.49, -2.5, -18.0, 29.0, -3.0, 15.0, 0.5854, 0.0,
            0.5757, 0.3350, 39.4387, -4.0864, -4.962, 1.5901, -42.5619, 21.2231, 0.0447, -0.7239,
            -2.2286, -0.5514,
        ];
        let d_e = vec![
            2.0425, 2.8615, 3.4412, 1.0, 1.0, 1.0, 2.3669, 2.3669, 7.1792, 7.1792, 7.2195, 7.2195,
            4.8045, 4.8045, 4.7461, 4.3065, 27.1492, 22.8977, 31.9030, 19.4535, 1.0, 1.0, 1.0, 1.0,
            1.2644, 1.263, 1.8731, 1.8645, 2.0373, 1.4146, 1.4441, 1.5381, 0.6377, 0.9082,
        ];
        assert_eq!(l_1.len(), 34);
        assert_eq!(l_2.len(), 34);
        assert_eq!(a_1.len(), 34);
        assert_eq!(a_2.len(), 34);
        assert_eq!(b_1.len(), 34);
        assert_eq!(b_2.len(), 34);
        assert_eq!(d_e.len(), 34);
        for i in 0..34 {
            let lab1 = CIELABColor {
                l: l_1[i],
                a: a_1[i],
                b: b_1[i],
            };
            let lab2 = CIELABColor {
                l: l_2[i],
                a: a_2[i],
                b: b_2[i],
            };
            // only good to 4 decimal points
            assert!((lab1.distance(&lab2) - d_e[i]).abs() <= 1e-4);
            assert!((lab2.distance(&lab1) - d_e[i]).abs() <= 1e-4);
        }
    }
    #[test]
    fn test_hue_chroma_lightness_saturation() {
        let mut rgb;
        let mut rgb2;
        for code in [
            "#12000D", "#FAFA22", "#FF0000", "#0000FF", "#FF0FDF", "#2266AA", "#001200", "#FFAAFF",
            "#003462", "#466223", "#AAFFBC",
        ].iter()
        {
            // hue
            rgb = RGBColor::from_hex_code(code).unwrap();
            let h = rgb.hue();
            rgb.set_hue(345.0);
            assert!((rgb.hue() - 345.0).abs() <= 1e-4);
            rgb2 = rgb;
            rgb2.set_hue(h);
            assert_eq!(rgb2.to_string(), String::from(*code));

            // chroma
            rgb = RGBColor::from_hex_code(code).unwrap();
            let c = rgb.chroma();
            rgb.set_chroma(45.0);
            assert!((rgb.chroma() - 45.0).abs() <= 1e-4);
            rgb2 = rgb;
            rgb2.set_chroma(c);
            assert_eq!(rgb2.to_string(), String::from(*code));

            // lightness
            rgb = RGBColor::from_hex_code(code).unwrap();
            let l = rgb.lightness();
            rgb.set_lightness(23.0);
            assert!((rgb.lightness() - 23.0).abs() <= 1e-4);
            rgb2 = rgb;
            rgb2.set_lightness(l);
            assert_eq!(rgb2.to_string(), String::from(*code));

            // saturation
            rgb = RGBColor::from_hex_code(code).unwrap();
            let s = rgb.saturation();
            rgb.set_saturation(0.4);
            assert!((rgb.saturation() - 0.4).abs() <= 1e-4);
            rgb2 = rgb;
            rgb2.set_saturation(s);
            assert_eq!(rgb2.to_string(), String::from(*code));
        }
    }    
}
