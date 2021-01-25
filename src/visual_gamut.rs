//! This file implements a rather complex and involved function: one that finds the closest color
//! visible by the human eye to a given color.
use color::XYZColor;
use illuminants::Illuminant;

use super::csv;

use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct Record {
    wavelength: u16,
    xbar: f64,
    ybar: f64,
    zbar: f64,
}

// first, read in spectral color data
pub fn read_cie_spectral_data() -> (Vec<u16>, Vec<XYZColor>) {
    let mut wavelengths = vec![];
    let mut xyz_data = vec![];
    let path = Path::new("cie-1931-standard-matching.csv");
    let mut reader = match csv::Reader::from_path(path) {
        Err(e) => panic!("CIE spectral data could not be read: {}", e.to_string()),
        Ok(rdr) => rdr,
    };
    for result in reader.deserialize() {
        // we should panic on bad data: this file is supplied by us!
        let record: Record = result.unwrap();
        wavelengths.push(record.wavelength);
        xyz_data.push(XYZColor {
            x: record.xbar,
            y: record.ybar,
            z: record.zbar,
            illuminant: Illuminant::D50,
        });
    }
    (wavelengths, xyz_data)
}
