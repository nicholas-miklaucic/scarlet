// This is still a preliminary file and won't make it into an actual version of Scarlet. It
// implements basic convenience functions that process valid HTML colors into vectors of either RGB,
// RGBA, HSL, or HSLA components.

// Takes in a hex string, such as "#23334412" or "#aabbcc", and returns an array of four color
// components R, G, B, and A as 0-255 integers. If the hex does not have an alpha value, it is
// assumed to be 255.
// This is not properly formatted for rustdoc because it doesn't fit within a larger framework yet.


use std::result::Result::Err;
use std::collections::HashMap;
extern crate regex;
use regex::Regex;


#[derive(Debug)]
enum ColorParseError {
    InvalidHTMLHex,
    InvalidRGBAFunction,
    OutOfRangeRGBA,
    OutOfRangeHSLA,
    InvalidHSLAFunction,
    InvalidX11Name
}

fn hex_to_rgba(hex: &str) -> Result<Vec<u8>, ColorParseError> {
    // Parse for valid hex: has to be of the form #rrggbb or #rrggbbaa, where the values after the
    // # have to be valid hex (0-9, a-f, or A-F)
    if hex.chars().nth(0) != Some('#') {
        return Err(ColorParseError::InvalidHTMLHex)
    }; 
    let count = hex.chars().count(); 
    if ![7, 9].contains(&count) {
        Err(ColorParseError::InvalidHTMLHex)
    } else if hex.chars().skip(1).any(|x| !("abcdefABCDEF0123456789".contains(x))) {
        Err(ColorParseError::InvalidHTMLHex)
    } else { // valid hex: return useful value
        // it's easier to do math on the hex value than it is to slice strings
        let mut full_num:u32 = u32::from_str_radix(hex.chars().skip(1).collect::<String>().as_str(), 16).unwrap();
        let mut a:u8 = 255;
        if count == 7 {
            // no need to do anything, alpha is already set
        }
        else {
            // alpha is the number modulo 16^2, divide the number by 16^2 to remove the alpha
            // component afterwards
            // this should become bit operations that are really fast, but I haven't checked it to
            // see if this actually happens, I'm just assuming for now that the compiler's good
            a = (full_num % 256) as u8;
            full_num = full_num / 256;
        }
        // now we just modulo and divide three more times, as we've guaranteed it's now rrggbb
        let b = (full_num % 256) as u8;
        full_num = full_num / 256;
        let g = (full_num % 256) as u8;
        full_num = full_num / 256;
        // the only thing left is now r
        let r = full_num as u8;
        Ok(vec![r, g, b, a])
    }
}

// Given a string like "rgb(24, 33, 235)" or "rgba(23, 34, 11)", returns a Vector [r, g, b, a].
// Returns a ColorParseError if the RGB is out of range or there is incorrect syntax.
fn func_to_rgba(input: &str) -> Result<Vec<u8>, ColorParseError> {
    // test for any characters that are not spaces, parentheses, commas,  the letters rgba, or digits
    if !(input.chars().all(|c| " ()rgba0123456789,".contains(c))) {
        return Err(ColorParseError::InvalidRGBAFunction)
    };
    // now we don't have to worry about indexing issues, as everything is ASCII
    // to simplify the string processing from here on out, we'll use regexes
    // this gives us the added benefit of parsing the numbers as well and runs in linear time
    // the tradeoff is that we won't know why the syntax is wrong, but it's not worth the 50 lines of
    // code it would take to accomplish that and the increased error likelihood
    // it's also a bit on the read-only side! testing is crucial here
    // note that this does not verify that the numbers are in bounds, just that they're between 0 and 999
    let rgba_re = Regex::new(r"rgba\((?: *(\d{1,3}),)(?: *(\d{1,3}),)(?: *(\d{1,3}),)(?: *(\d{1,3}))\)").unwrap();
    let rgb_re = Regex::new(r"rgb\((?: *(\d{1,3}),)(?: *(\d{1,3}),)(?: *(\d{1,3}))\)").unwrap();
   
    // try matching each one, falling back to rgb if rgba fails
    let grps = match rgba_re.captures(input) {
        None => rgb_re.captures(input),
        x @ Some(_) => x
    };
    // now either return an error, process an RGBA string list, or an RGB string list
    match grps {
        None => Err(ColorParseError::InvalidRGBAFunction),
        Some(ref x) if x.len() == 4 => {
            let (rstr, gstr, bstr) = (&x[1], &x[2], &x[3]);
            let a:u8 = 255;
            // use u16s so that values > 255 will be detected
            let r = u16::from_str_radix(rstr, 10).unwrap();
            let g = u16::from_str_radix(gstr, 10).unwrap();
            let b = u16::from_str_radix(bstr, 10).unwrap();
            if (r > 255) | (g > 255) | (b > 255) {
                Err(ColorParseError::OutOfRangeRGBA)
            }
            else {
                Ok(vec![r as u8, g as u8, b as u8, a])
            }
        }
        Some(ref x) if x.len() == 5 => {
            let (rstr, gstr, bstr, astr) = (&x[1], &x[2], &x[3], &x[4]);
            let r = u16::from_str_radix(rstr, 10).unwrap();
            let g = u16::from_str_radix(gstr, 10).unwrap();
            let b = u16::from_str_radix(bstr, 10).unwrap();
            let a = u16::from_str_radix(astr, 10).unwrap();
            if (r > 255) | (g > 255) | (b > 255) | (a > 255) {
                Err(ColorParseError::OutOfRangeRGBA)
            }
            else {
                Ok(vec![r as u8, g as u8, b as u8, a as u8])
            }
        },
        Some(_) => Err(ColorParseError::InvalidRGBAFunction)
    }
}

// Takes in an X11 color name, like "blue", and returns the corresponding 
// RGB values as a vector [r, g, b, a] where a is 255.
fn name_to_rgba(name: &str) -> Result<Vec<u8>, ColorParseError> {
    // this is the full list of X11 color names
    // I used a Python script to process it from this site:
    // https://github.com/bahamas10/css-color-names/blob/master/css-color-names.json let
    // I added the special "transparent" referring to #00000000
    let color_names:Vec<&str> = [
        "aliceblue", "antiquewhite", "aqua", "aquamarine", "azure", "beige",
        "bisque", "black", "blanchedalmond", "blue", "blueviolet", "brown", "burlywood", "cadetblue",
        "chartreuse", "chocolate", "coral", "cornflowerblue", "cornsilk", "crimson", "cyan", "darkblue",
        "darkcyan", "darkgoldenrod", "darkgray", "darkgreen", "darkgrey", "darkkhaki", "darkmagenta",
        "darkolivegreen", "darkorange", "darkorchid", "darkred", "darksalmon", "darkseagreen",
        "darkslateblue", "darkslategray", "darkslategrey", "darkturquoise", "darkviolet", "deeppink",
        "deepskyblue", "dimgray", "dimgrey", "dodgerblue", "firebrick", "floralwhite", "forestgreen",
        "fuchsia", "gainsboro", "ghostwhite", "gold", "goldenrod", "gray", "green", "greenyellow",
        "grey", "honeydew", "hotpink", "indianred", "indigo", "ivory", "khaki", "lavender",
        "lavenderblush", "lawngreen", "lemonchiffon", "lightblue", "lightcoral", "lightcyan",
        "lightgoldenrodyellow", "lightgray", "lightgreen", "lightgrey", "lightpink", "lightsalmon",
        "lightseagreen", "lightskyblue", "lightslategray", "lightslategrey", "lightsteelblue",
        "lightyellow", "lime", "limegreen", "linen", "magenta", "maroon", "mediumaquamarine",
        "mediumblue", "mediumorchid", "mediumpurple", "mediumseagreen", "mediumslateblue",
        "mediumspringgreen", "mediumturquoise", "mediumvioletred", "midnightblue", "mintcream",
        "mistyrose", "moccasin", "navajowhite", "navy", "oldlace", "olive", "olivedrab", "orange",
        "orangered", "orchid", "palegoldenrod", "palegreen", "paleturquoise", "palevioletred",
        "papayawhip", "peachpuff", "peru", "pink", "plum", "powderblue", "purple", "rebeccapurple",
        "red", "rosybrown", "royalblue", "saddlebrown", "salmon", "sandybrown", "seagreen", "seashell",
        "sienna", "silver", "skyblue", "slateblue", "slategray", "slategrey", "snow", "springgreen",
        "steelblue", "tan", "teal", "thistle", "tomato", "turquoise", "violet", "wheat", "white",
        "whitesmoke", "yellow", "yellowgreen", "transparent"
    ].to_vec();
    let color_codes:Vec<&str> = [
        "#f0f8ff", "#faebd7", "#00ffff", "#7fffd4", "#f0ffff", "#f5f5dc", "#ffe4c4", "#000000",
        "#ffebcd", "#0000ff", "#8a2be2", "#a52a2a", "#deb887", "#5f9ea0", "#7fff00", "#d2691e",
        "#ff7f50", "#6495ed", "#fff8dc", "#dc143c", "#00ffff", "#00008b", "#008b8b", "#b8860b",
        "#a9a9a9", "#006400", "#a9a9a9", "#bdb76b", "#8b008b", "#556b2f", "#ff8c00", "#9932cc",
        "#8b0000", "#e9967a", "#8fbc8f", "#483d8b", "#2f4f4f", "#2f4f4f", "#00ced1", "#9400d3",
        "#ff1493", "#00bfff", "#696969", "#696969", "#1e90ff", "#b22222", "#fffaf0", "#228b22",
        "#ff00ff", "#dcdcdc", "#f8f8ff", "#ffd700", "#daa520", "#808080", "#008000", "#adff2f",
        "#808080", "#f0fff0", "#ff69b4", "#cd5c5c", "#4b0082", "#fffff0", "#f0e68c", "#e6e6fa",
        "#fff0f5", "#7cfc00", "#fffacd", "#add8e6", "#f08080", "#e0ffff", "#fafad2", "#d3d3d3",
        "#90ee90", "#d3d3d3", "#ffb6c1", "#ffa07a", "#20b2aa", "#87cefa", "#778899", "#778899",
        "#b0c4de", "#ffffe0", "#00ff00", "#32cd32", "#faf0e6", "#ff00ff", "#800000", "#66cdaa",
        "#0000cd", "#ba55d3", "#9370db", "#3cb371", "#7b68ee", "#00fa9a", "#48d1cc", "#c71585",
        "#191970", "#f5fffa", "#ffe4e1", "#ffe4b5", "#ffdead", "#000080", "#fdf5e6", "#808000",
        "#6b8e23", "#ffa500", "#ff4500", "#da70d6", "#eee8aa", "#98fb98", "#afeeee", "#db7093",
        "#ffefd5", "#ffdab9", "#cd853f", "#ffc0cb", "#dda0dd", "#b0e0e6", "#800080", "#663399",
        "#ff0000", "#bc8f8f", "#4169e1", "#8b4513", "#fa8072", "#f4a460", "#2e8b57", "#fff5ee",
        "#a0522d", "#c0c0c0", "#87ceeb", "#6a5acd", "#708090", "#708090", "#fffafa", "#00ff7f",
        "#4682b4", "#d2b48c", "#008080", "#d8bfd8", "#ff6347", "#40e0d0", "#ee82ee", "#f5deb3",
        "#ffffff", "#f5f5f5", "#ffff00", "#9acd32", "#00000000"
    ].to_vec();
    let mut names_to_codes = HashMap::new();

    for (i, color_name) in color_names.iter().enumerate() {
        names_to_codes.insert(color_name, color_codes[i]);
    }

    // now just return the converted value or raise one if not in hashmap
    match names_to_codes.get(&name) {
        None => Err(ColorParseError::InvalidX11Name),
        Some(x) => hex_to_rgba(x)
    }
}

// Now, we begin the public part of this: defining the structs that we'll use to represent different
// colors in different color spaces.

// First, we define a trait that identifies a color as a color: for our purposes, this will be some
// way of converting to and from a color in the CIE XYZ space (along with an associated luminant).

pub trait Chromatic {
    

pub struct RGBColor {
    r: u8;
    g: u8;
    b: u8;
    unbounded_r: f32;
    unbounded_g: f32;
    unbounded_b: f32;
}

impl RGBColor {
    pub fn new(r: u8, g: u8, b: u8) -> RGBColor {
        RGBColor {
            r,
            g,
            b,
            unbounded_r: r,
            unbounded_g: g,
            unbounded_b: b
        }
  
}
