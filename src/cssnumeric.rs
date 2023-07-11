//! This file separates out the more difficult aspects of string parsing, in this case dealing with
//! CSS numeric notation and all of its warts. Its goal is to, at the end, provide a function to
//! encode arbitrary CSS color descriptions into Scarlet structs. (Source for CSS syntax:
//! [https://www.w3.org/TR/css-color-3/](https://www.w3.org/TR/css-color-3/).)

use std::error::Error;
use std::fmt;

/// A CSS numeric value. Either an integer, like 255, a float, like 0.8, or a percentage, like
/// 104%.
#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum CSSNumeric {
    /// Represents a string of numeric tokens, such as "124", with an optional leading '+' or '-'.
    Integer(isize),
    /// Represents two integers separated by a '.', such that the second integer has no leading sign.
    Float(f64),
    /// Represents an integer followed by '%', to denote one one-hundredth of that integer.
    Percentage(isize),
}

/// An error in parsing a CSS string. Covers many different kinds of errors.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum CSSParseError {
    /// This indicates that non-numeric characters were used in a string on which a parse into a
    /// number was attempted.
    InvalidNumericCharacters,
    /// This indicates that invalid numeric syntax was used, such as multiple periods or plus or minus
    /// in invalid places.
    InvalidNumericSyntax,
    /// This indicates that a general color syntax error occurred, such as mismatching parentheses or
    /// uninterpretable tokens.
    InvalidColorSyntax,
}

impl fmt::Display for CSSParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CSS parsing error")
    }
}

impl Error for CSSParseError {
    fn description(&self) -> &str {
        match *self {
            CSSParseError::InvalidNumericCharacters => "Unexpected non-numeric characters",
            CSSParseError::InvalidNumericSyntax => "Invalid numeric syntax",
            CSSParseError::InvalidColorSyntax => "Invalid color syntax",
        }
    }
}

/// Parses a prechecked integer without a sign, such as "023" or "142". Panics on invalid input.
fn parse_css_integer(num: &str) -> u64 {
    num.parse().unwrap()
}

/// Parses a CSS float, such as "123.42" or ".34". Panics on invalid input.
fn parse_css_float(num: &str) -> f64 {
    num.parse().unwrap()
}

/// Parses a given CSS float (two integers separated by '.'), CSS integer (a string of characters
/// '0'-'9') or a CSS percentage (an integer followed by '%'). Returns a struct that represents these
/// various possibilities.
pub(crate) fn parse_css_number(num: &str) -> Result<CSSNumeric, CSSParseError> {
    let mut chars: Vec<char> = num.chars().collect();
    // if invalid characters, return appropriate error
    if !chars.iter().all(|&c| "0123456789-+.%".contains(c)) {
        return Err(CSSParseError::InvalidNumericCharacters);
    }
    // test if initial character is '-' or '+'. Remove and set sign flag accordingly.
    let is_positive = match chars[0] {
        '-' => false,
        '+' => true,
        _ => true,
    };
    if "-+".contains(chars[0]) {
        chars.remove(0);
    }
    // if no longer any characters, throw error
    if chars.is_empty() {
        return Err(CSSParseError::InvalidNumericSyntax);
    }
    // if any other pluses or minuses, throw error
    if chars.iter().any(|&c| "-=".contains(c)) {
        return Err(CSSParseError::InvalidNumericSyntax);
    }
    // Test if number contains exactly one period. If more than one, throw error: otherwise, split to
    // cases.
    match chars.iter().filter(|&c| c == &'.').count() {
        0 => {
            // number or percentage: check, throw error if more than one %
            match chars.iter().filter(|&c| c == &'%').count() {
                0 => {
                    // well-formed integer
                    let uint = parse_css_integer(&(chars.iter().collect::<String>()));
                    // adjust for sign
                    let int = if is_positive {
                        uint as isize
                    } else {
                        -(uint as isize)
                    };
                    Ok(CSSNumeric::Integer(int))
                }
                1 => {
                    // check if % is at end
                    if chars.iter().last().unwrap() == &'%' {
                        // parse the rest as integer and return
                        chars.pop();
                        let uint = parse_css_integer(&(chars.iter().collect::<String>()));
                        // adjust for sign
                        let int = if is_positive {
                            uint as isize
                        } else {
                            -(uint as isize)
                        };
                        Ok(CSSNumeric::Percentage(int))
                    } else {
                        // invalid, throw error
                        Err(CSSParseError::InvalidNumericSyntax)
                    }
                }
                _ => {
                    // invalid, throw eerror
                    Err(CSSParseError::InvalidNumericSyntax)
                }
            }
        }
        1 => {
            // parse as valid float and account for sign
            let ufloat = parse_css_float(&(chars.iter().collect::<String>()));
            let float = if is_positive { ufloat } else { -ufloat };
            Ok(CSSNumeric::Float(float))
        }
        _ => {
            // invalid, throw error
            Err(CSSParseError::InvalidNumericSyntax)
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_css_parse_integer() {
        let num1 = match parse_css_number("184").unwrap() {
            CSSNumeric::Integer(val) => val,
            _ => 0,
        };
        assert_eq!(num1, 184);
        // test leading zeros
        let num2 = match parse_css_number("00423").unwrap() {
            CSSNumeric::Integer(val) => val,
            _ => 0,
        };
        assert_eq!(num2, 423);
        // test negative sign
        let num3 = match parse_css_number("-00423").unwrap() {
            CSSNumeric::Integer(val) => val,
            _ => 0,
        };
        assert_eq!(num3, -423);
        // test positive sign
        let num4 = match parse_css_number("+00423").unwrap() {
            CSSNumeric::Integer(val) => val,
            _ => 0,
        };
        assert_eq!(num4, 423);
    }
    #[test]
    fn test_css_parse_float() {
        let num1 = match parse_css_number("0.37").unwrap() {
            CSSNumeric::Float(val) => val,
            _ => 0.,
        };
        assert_eq!(format!("{}", num1), "0.37");
        // test no leading zeros
        let num2 = match parse_css_number(".423").unwrap() {
            CSSNumeric::Float(val) => val,
            _ => 0.,
        };
        assert_eq!(format!("{}", num2), "0.423");
        // test negative sign
        let num3 = match parse_css_number("-00.423").unwrap() {
            CSSNumeric::Float(val) => val,
            _ => 0.,
        };
        assert_eq!(format!("{}", num3), "-0.423");
        // test positive sign
        let num4 = match parse_css_number("+00.423").unwrap() {
            CSSNumeric::Float(val) => val,
            _ => 0.,
        };
        assert_eq!(format!("{}", num4), "0.423");
    }
    #[test]
    fn test_css_parse_percentages() {
        // just repeating integer tests for this one
        let num1 = match parse_css_number("184%").unwrap() {
            CSSNumeric::Percentage(val) => val,
            _ => 0,
        };
        assert_eq!(num1, 184);
        // test leading zeros
        let num2 = match parse_css_number("00423%").unwrap() {
            CSSNumeric::Percentage(val) => val,
            _ => 0,
        };
        assert_eq!(num2, 423);
        // test negative sign
        let num3 = match parse_css_number("-00423%").unwrap() {
            CSSNumeric::Percentage(val) => val,
            _ => 0,
        };
        assert_eq!(num3, -423);
        // test positive sign
        let num4 = match parse_css_number("+00423%").unwrap() {
            CSSNumeric::Percentage(val) => val,
            _ => 0,
        };
        assert_eq!(num4, 423);
    }
    #[test]
    fn test_errors() {
        // test non-numeric characters
        assert_eq!(
            parse_css_number("abc"),
            Err(CSSParseError::InvalidNumericCharacters)
        );
        // test multiple periods
        assert_eq!(
            parse_css_number("14.23.2"),
            Err(CSSParseError::InvalidNumericSyntax)
        );
        // test multiple percentages, percentages in wrong place
        assert_eq!(
            parse_css_number("-24%%"),
            Err(CSSParseError::InvalidNumericSyntax)
        );
        assert_eq!(
            parse_css_number("1%2%"),
            Err(CSSParseError::InvalidNumericSyntax)
        );
    }
}
