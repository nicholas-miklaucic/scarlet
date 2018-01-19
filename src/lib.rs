struct RGBColor {
    r: u8;
    g: u8;
    b: u8;
    unbounded_r: f32;
    unbounded_g: f32;
    unbounded_b: f32;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
