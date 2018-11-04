//! Scarlet is a library for making color, color spaces, and everything that comes with it simple to
//! work with. The underlying philosophy is that if all you have is a hammer, everything looks like a
//! nail: existing color libraries often only work with RGB or other convenient color spaces, and so
//! go to great lengths to invent complicated workarounds for the essential problems with RGB and its
//! ilk, namely not being very good analogues to the way humans actually see color. Scarlet makes
//! working with color convenient enough that it's *easier* to treat colors correctly than it is to do
//! anything else.

#![doc(html_root_url = "https://docs.rs/scarlet/1.0.2")]

// we don't mess around with documentation
#![deny(missing_docs)]
// Clippy doesn't like long decimals, but adding separators in decimals isn't any more readable
// compare -0.96924 with -0.96_924
#![allow(clippy::unreadable_literal)]

extern crate csv;
extern crate geo;
#[macro_use]
extern crate rulinalg;
extern crate num;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate termion;
#[macro_use]
extern crate lazy_static;

pub mod coord;
mod consts;
mod matplotlib_cmaps;
mod cssnumeric;
mod csscolor;
pub mod illuminants;
pub mod color;
pub mod bound;
mod visual_gamut;
pub mod colors;
pub mod colorpoint;
pub mod colormap;
pub mod material_colors;
pub mod prelude;
// pub mod doc;



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
