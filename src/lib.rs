extern crate csv;
extern crate geo;
extern crate nalgebra as na;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate termion;

pub mod coord;
pub mod consts;
pub mod matplotlib_cmaps;
pub mod illuminants;
pub mod color;
pub mod bound;
mod visual_gamut;
pub mod colors;
pub mod colorpoint;
pub mod colormap;
pub mod material_colors;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
