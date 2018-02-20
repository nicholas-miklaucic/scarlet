//! This file provides some basic facilities for creating a Color object, specifically an RGBColor,
//! from the Google Material design spec.

use color::{RGBColor};


/// A neutral tint or shade of a given Material Design hue. Although the values are usually given as
/// numerical literals, numerical literals are not valid identifiers.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum NeutralTone {
    W50,
    W100,
    W200,
    W300,
    W400,
    W500,
    W600,
    W700,
    W800,
    W900,
}

/// An accent tone, notated with an A prefix in the Material Design document and here.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum AccentTone {
    A100,
    A200,
    A400,
    A700,
}


/// Either a neutral or accent tone, with a prefix to distinguish them.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum MaterialTone {
    Neutral(NeutralTone),
    Accent(AccentTone),
}

// Gets the index in the list returned by the hashmap corresponding to the given tone.
fn to_index(tone: MaterialTone) -> usize {
    match tone {
        MaterialTone::Neutral(w) => match w {
            NeutralTone::W500 => 0,
            NeutralTone::W50 => 1,
            NeutralTone::W100 => 2,
            NeutralTone::W200 => 3,
            NeutralTone::W300 => 4,
            NeutralTone::W400 => 5,
            NeutralTone::W600 => 6,
            NeutralTone::W700 => 7,
            NeutralTone::W800 => 8,
            NeutralTone::W900 => 9,
        },
        MaterialTone::Accent(a) => match a {
            AccentTone::A100 => 10,
            AccentTone::A200 => 11,
            AccentTone::A400 => 12,
            AccentTone::A700 => 13,
        }
    }
}


#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
/// A fully specified Material color. Note that, depending on whether the color has accent tones,
/// different tone enums need to be used.
pub enum MaterialPrimary {
    Red(MaterialTone),
    Pink(MaterialTone),
    Purple(MaterialTone),
    DeepPurple(MaterialTone),
    Indigo(MaterialTone),
    Blue(MaterialTone),
    LightBlue(MaterialTone),
    Cyan(MaterialTone),
    Teal(MaterialTone),
    Green(MaterialTone),
    LightGreen(MaterialTone),
    Lime(MaterialTone),
    Yellow(MaterialTone),
    Amber(MaterialTone),
    Orange(MaterialTone),
    DeepOrange(MaterialTone),
    Brown(NeutralTone),
    Grey(NeutralTone),
    BlueGrey(NeutralTone),
    Black,
    White,
}



// values copied from material-palette.csv, which is in turn copied from the Material Design
// Photoshop palette
const RED_COLORS: [&str; 14] = ["#f44336","#ffebee","#ffcdd2","#ef9a9a","#e57373","#ef5350","#e53935","#d32f2f","#c62828","#b71c1c","#ff8a80","#ff5252","#ff1744","#d50000"];
const PINK_COLORS: [&str; 14] = ["#e91e63","#fce4ec","#f8bbd0","#f48fb1","#f06292","#ec407a","#d81b60","#c2185b","#ad1457","#880e4f","#ff80ab","#ff4081","#f50057","#c51162"];
const PURPLE_COLORS: [&str; 14] = ["#9c27b0","#f3e5f5","#e1bee7","#ce93d8","#ba68c8","#ab47bc","#8e24aa","#7b1fa2","#6a1b9a","#4a148c","#ea80fc","#e040fb","#d500f9","#aa00ff"];
const DEEP_PURPLE_COLORS: [&str; 14] = ["#673ab7","#ede7f6","#d1c4e9","#b39ddb","#9575cd","#7e57c2","#5e35b1","#512da8","#4527a0","#311b92","#b388ff","#7c4dff","#651fff","#6200ea"];
const INDIGO_COLORS: [&str; 14] = ["#3f51b5","#e8eaf6","#c5cae9","#9fa8da","#7986cb","#5c6bc0","#3949ab","#303f9f","#283593","#1a237e","#8c9eff","#536dfe","#3d5afe","#304ffe"];
const BLUE_COLORS: [&str; 14] = ["#2196f3","#e3f2fd","#bbdefb","#90caf9","#64b5f6","#42a5f5","#1e88e5","#1976d2","#1565c0","#0d47a1","#82b1ff","#448aff","#2979ff","#2962ff"];
const LIGHT_BLUE_COLORS: [&str; 14] = ["#03a9f4","#e1f5fe","#b3e5fc","#81d4fa","#4fc3f7","#29b6f6","#039be5","#0288d1","#0277bd","#01579b","#80d8ff","#40c4ff","#00b0ff","#0091ea"];
const CYAN_COLORS: [&str; 14] = ["#00bcd4","#e0f7fa","#b2ebf2","#80deea","#4dd0e1","#26c6da","#00acc1","#0097a7","#00838f","#006064","#84ffff","#18ffff","#00e5ff","#00b8d4"];
const TEAL_COLORS: [&str; 14] = ["#009688","#e0f2f1","#b2dfdb","#80cbc4","#4db6ac","#26a69a","#00897b","#00796b","#00695c","#004d40","#a7ffeb","#64ffda","#1de9b6","#00bfa5"];
const GREEN_COLORS: [&str; 14] = ["#4caf50","#e8f5e9","#c8e6c9","#a5d6a7","#81c784","#66bb6a","#43a047","#388e3c","#2e7d32","#1b5e20","#b9f6ca","#69f0ae","#00e676","#00c853"];
const LIGHT_GREEN_COLORS: [&str; 14] = ["#8bc34a","#f1f8e9","#dcedc8","#c5e1a5","#aed581","#9ccc65","#7cb342","#689f38","#558b2f","#33691e","#ccff90","#b2ff59","#76ff03","#64dd17"];
const LIME_COLORS: [&str; 14] = ["#cddc39","#f9fbe7","#f0f4c3","#e6ee9c","#dce775","#d4e157","#c0ca33","#afb42b","#9e9d24","#827717","#f4ff81","#eeff41","#c6ff00","#aeea00"];
const YELLOW_COLORS: [&str; 14] = ["#ffeb3b","#fffde7","#fff9c4","#fff59d","#fff176","#ffee58","#fdd835","#fbc02d","#f9a825","#f57f17","#ffff8d","#ffff00","#ffea00","#ffd600"];
const AMBER_COLORS: [&str; 14] = ["#ffc107","#fff8e1","#ffecb3","#ffe082","#ffd54f","#ffca28","#ffb300","#ffa000","#ff8f00","#ff6f00","#ffe57f","#ffd740","#ffc400","#ffab00"];
const ORANGE_COLORS: [&str; 14] = ["#ff9800","#fff3e0","#ffe0b2","#ffcc80","#ffb74d","#ffa726","#fb8c00","#f57c00","#ef6c00","#e65100","#ffd180","#ffab40","#ff9100","#ff6d00"];
const DEEP_ORANGE_COLORS: [&str; 14] = ["#ff5722","#fbe9e7","#ffccbc","#ffab91","#ff8a65","#ff7043","#f4511e","#e64a19","#d84315","#bf360c","#ff9e80","#ff6e40","#ff3d00","#dd2c00"];
const GREY_COLORS: [&str; 10] = ["#9e9e9e","#fafafa","#f5f5f5","#eeeeee","#e0e0e0","#bdbdbd","#757575","#616161","#424242","#212121"];
const BLUE_GREY_COLORS: [&str; 10] = ["#607d8b","#eceff1","#cfd8dc","#b0bec5","#90a4ae","#78909c","#546e7a","#455a64","#37474f","#263238"];
const BROWN_COLORS: [&str; 10] = ["#795548","#efebe9","#d7ccc8","#bcaaa4","#a1887f","#8d6e63","#6d4c41","#5d4037","#4e342e","#3e2723"];



impl RGBColor {
    /// Gets a Color from the Material palette, given a specification of such a color.
    pub fn from_material_palette(prim: MaterialPrimary) -> RGBColor {
        // get hex code
        let hex_code = match prim {
            MaterialPrimary::Red(mat) => RED_COLORS[to_index(mat)],
            MaterialPrimary::Pink(mat) => PINK_COLORS[to_index(mat)],
            MaterialPrimary::Purple(mat) => PURPLE_COLORS[to_index(mat)],
            MaterialPrimary::DeepPurple(mat) => DEEP_PURPLE_COLORS[to_index(mat)],
            MaterialPrimary::Indigo(mat) => INDIGO_COLORS[to_index(mat)],
            MaterialPrimary::Blue(mat) => BLUE_COLORS[to_index(mat)],
            MaterialPrimary::LightBlue(mat) => LIGHT_BLUE_COLORS[to_index(mat)],
            MaterialPrimary::Cyan(mat) => CYAN_COLORS[to_index(mat)],
            MaterialPrimary::Teal(mat) => TEAL_COLORS[to_index(mat)],
            MaterialPrimary::Green(mat) => GREEN_COLORS[to_index(mat)],
            MaterialPrimary::LightGreen(mat) => LIGHT_GREEN_COLORS[to_index(mat)],
            MaterialPrimary::Lime(mat) => LIME_COLORS[to_index(mat)],
            MaterialPrimary::Yellow(mat) => YELLOW_COLORS[to_index(mat)],
            MaterialPrimary::Amber(mat) => AMBER_COLORS[to_index(mat)],
            MaterialPrimary::Orange(mat) => ORANGE_COLORS[to_index(mat)],
            MaterialPrimary::DeepOrange(mat) => DEEP_ORANGE_COLORS[to_index(mat)],
            MaterialPrimary::Brown(neut) => BROWN_COLORS[to_index(MaterialTone::Neutral(neut))],
            MaterialPrimary::Grey(neut) => GREY_COLORS[to_index(MaterialTone::Neutral(neut))],
            MaterialPrimary::BlueGrey(neut) => BLUE_GREY_COLORS[to_index(MaterialTone::Neutral(neut))],
            MaterialPrimary::Black => "#000000",
            MaterialPrimary::White => "#ffffff",
        };
        // guaranteed to be valid, so unwrapping is fine: panicking indicates a bug
        RGBColor::from_hex_code(hex_code).unwrap()
    }
}


#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_sample_colors() {
        // just a couple random ones, to test the general process
        let blue50 = MaterialPrimary::Blue(MaterialTone::Neutral(NeutralTone::W50));
        let red_a100 = MaterialPrimary::Red(MaterialTone::Accent(AccentTone::A100));
        let bluegrey400 = MaterialPrimary::BlueGrey(NeutralTone::W400);
        let black = MaterialPrimary::Black;
        let green500 = MaterialPrimary::Green(MaterialTone::Neutral(NeutralTone::W500));
        
        assert_eq!(RGBColor::from_material_palette(blue50).to_string(), "#E3F2FD");
        assert_eq!(RGBColor::from_material_palette(red_a100).to_string(), "#FF8A80");
        assert_eq!(RGBColor::from_material_palette(bluegrey400).to_string(), "#78909C");
        assert_eq!(RGBColor::from_material_palette(black).to_string(), "#000000");
        assert_eq!(RGBColor::from_material_palette(green500).to_string(), "#4CAF50");
    }
}
