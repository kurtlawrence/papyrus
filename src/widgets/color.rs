use azul::prelude::*;

pub const fn map(color: cansi::Color) -> ColorU {
    use cansi::Color as cc;
    match color {
        cc::Black => BLACK,
		cc::Red => RED,
        cc::Cyan => CYAN,
    }
}

const fn coloru(r: u8, b: u8, g: u8) -> ColorU {
    ColorU {
        r: r,
        b: b,
        g: g,
        a: 255,
    }
}

const BLACK: ColorU = coloru(0, 0, 0);
const RED: ColorU = coloru(170, 0, 0);
const CYAN: ColorU = coloru(0, 170, 170);