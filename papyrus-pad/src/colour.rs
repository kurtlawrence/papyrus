use super::*;

macro_rules! rgb {
    ($r:expr, $g:expr, $b:expr) => {{
        ColorU {
            r: $r,
            g: $g,
            b: $b,
            a: 255,
        }
    }};
}

pub fn map(color: &cansi::Color) -> ColorU {
    use cansi::Color as cc;
    match color {
        cc::Black => rgb!(1, 1, 1),
        cc::Red => rgb!(222, 56, 43),
        cc::Green => rgb!(57, 181, 74),
        cc::Yellow => rgb!(255, 199, 6),
        cc::Blue => rgb!(0, 111, 184),
        cc::Magenta => rgb!(118, 38, 113),
        cc::Cyan => rgb!(44, 181, 233),
        cc::White => rgb!(204, 204, 204),
        cc::BrightBlack => rgb!(128, 128, 128),
        cc::BrightRed => rgb!(255, 0, 0),
        cc::BrightGreen => rgb!(0, 255, 0),
        cc::BrightYellow => rgb!(255, 255, 0),
        cc::BrightBlue => rgb!(0, 0, 255),
        cc::BrightMagenta => rgb!(255, 0, 255),
        cc::BrightCyan => rgb!(0, 255, 255),
        cc::BrightWhite => rgb!(255, 255, 225),
    }
}
