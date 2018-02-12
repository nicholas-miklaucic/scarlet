//! This file provides constants that are used for matrix multiplication and color space conversion,
//! along with a function for computing inverses. The reason for this method of doing things instead
//! of simple multiplications and additions is because the inverses of these transformations become
//! slightly off, allowing for errors to slowly creep in even when doing things that should not
//! change the result at all, e.g., converting to an illuminant and back again. Thus, this method
//! allows for saner checking of constant values and guaranteed precision in inversion.

use na::Matrix3;

/// Not safe for general use. If `const fn` was stable, it would be used instead. The only reason this
/// is here is to calculate the inverse of constant matrices. This panics on singular matrices!
pub fn inv(m: Matrix3<f64>) -> Matrix3<f64> {
    if !m.is_invertible() {
        panic!("Constant matrix not invertible!")
    } else {
        m.try_inverse().unwrap()
    }
}

#[allow(non_snake_case)]
pub fn ADOBE_RGB_TRANSFORM_MAT() -> Matrix3<f64> {
    Matrix3::new(
        02.04159,
        -0.56501,
        -0.34473,
        -0.96924,
        01.87957,
        00.04156,
        00.01344,
        -0.11836,
        01.01517,
    )
}

#[allow(non_snake_case)]
pub fn BRADFORD_TRANSFORM_MAT() -> Matrix3<f64> {
    Matrix3::new(
        00.8951,
        00.2664,
        -0.1614,
        -0.7502,
        01.7135,
        00.0367,
        00.0389,
        -0.0685,
        01.0296,
    )
}

#[allow(non_snake_case)]
pub fn ROMM_RGB_TRANSFORM_MAT() -> Matrix3<f64> {
    Matrix3::new(
        0.7976749,
        0.1351917,
        0.0313534,
        0.2880402,
        0.7118741,
        0.0000857,
        0.0000000,
        0.0000000,
        0.8252100,
    )
}

#[allow(non_snake_case)]
pub fn STANDARD_RGB_TRANSFORM_MAT() -> Matrix3<f64> {
    Matrix3::new(
        03.2406,
        -1.5372,
        -0.4986,
        -0.9689,
        01.8758,
        00.0415,
        00.0557,
        -0.2040,
        01.0570,
    )
}
