extern crate csv;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate geo;


pub mod coord;
pub mod illuminants;
pub mod color;
mod visual_gamut;
pub mod colors;
pub mod color_funcs;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
