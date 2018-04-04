//! This module contains various modules that implement types that implement [`Color`]. For convenience,
//! each main type is imported into this module's namespace directly.
//!
//! [`Color`]: ../color/trait.Color.html
pub mod adobergbcolor;
pub mod cielabcolor;
pub mod cielchcolor;
pub mod cieluvcolor;
pub mod cielchuvcolor;
pub mod hslcolor;
pub mod hsvcolor;
pub mod rommrgbcolor;


// for convenience, use this namespace for the color objects
pub use self::adobergbcolor::AdobeRGBColor;
pub use self::cielabcolor::CIELABColor;
pub use self::cielchcolor::CIELCHColor;
pub use self::cieluvcolor::CIELUVColor;
pub use self::cielchuvcolor::CIELCHuvColor;
pub use self::hslcolor::HSLColor;
pub use self::hsvcolor::HSVColor;
pub use self::rommrgbcolor::ROMMRGBColor;
