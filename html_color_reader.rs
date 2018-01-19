// This is still a preliminary file and won't make it into an actual version of Scarlet. It
// implements basic convenience functions that process valid HTML colors into vectors of either RGB,
// RGBA, HSL, or HSLA components.

// Takes in a hex string, such as "#23334412" or "#aabbcc", and returns an array of four color
// components R, G, B, and A as 0-255 integers. If the hex does not have an alpha value, it is
// assumed to be 255.
// This is not properly formatted for rustdoc because it doesn't fit within a larger framework yet.


use std::result::Result::Err;
extern crate regex;
use regex::Regex;

pub enum ColorParseError {
    InvalidHTMLHex,
    InvalidRGBAFunction,
    OutOfRangeRGBA,
    OutOfRangeHSLA,
    InvalidHSLAFunction
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
    rgba_re = Regex::new(r"^rgba\((?: *([0-9]{1, 3}) *,\)){4}$").unwrap();
    rgb_re = Regex::new(r"^rgb\((?: *([0-9]{1, 3}) *,\)){3}$").unwrap();
    // try matching each one, falling back to rgb if rgba fails
    let grps:Option<Captures<'t>> = match rgba_re.captures(input) {
        None => match rgb_re.captures(input) {
            None => None,
            Some(x) => Some(x)
        },
        Some(x) => Some(x)
    };
    // now either return an error, process an RGBA string list, or an RGB string list
    match grps {
        None => Err(ColorParseError::InvalidRGBAFunction),
        Some(x) if x.count() == 3 => {
            let (rstr, gstr, bstr) = x.collect();
            let a:u8 = 255;
            // use u16s so that values > 255 will be detected
            let r = u16::from_str_radix(rstr.as_str(), 10)?;
            let g = u16::from_str_radix(gstr.as_str(), 10)?;
            let b = u16::from_str_radix(bstr.as_str(), 10)?;
            if (r > 255) | (g > 255) | (b > 255) {
                Err(ColorParseError::OutOfRangeRGBA)
            }
            else {
                Ok(vec![r as u8, g as u8, b as u8, a])
            }
        }
        Some(x) if x.count() == 4 => {
            let (rstr, gstr, bstr, astr) = x.collect();
            let r = u16::from_str_radix(rstr.as_str(), 10)?;
            let g = u16::from_str_radix(gstr.as_str(), 10)?;
            let b = u16::from_str_radix(bstr.as_str(), 10)?;
            let a = u16::from_str_radix(astr.as_str(), 10)?;
            if (r > 255) | (g > 255) | (b > 255) | (a > 255) {
                Err(ColorParseError::OutOfRangeRGBA)
            }
            else {
                Ok(vec![r as u8, g as u8, b as u8, a as u8])
            }
        }
    }
}
    

fn main() {
    println!("{:?}", hex_to_rgba(&"#ff0080".to_string()).ok());
    println!("{:?}", func_to_rgba(&"rgba(255, 0, 128, 56)".to_string()).ok());
}
