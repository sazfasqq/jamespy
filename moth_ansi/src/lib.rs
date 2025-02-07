use aformat::aformat;

pub const RESET: &str = "\x1B[0m";
pub const BOLD: &str = "\x1B[1m";
pub const DIM: &str = "\x1B[2m";

pub const RED: &str = "\x1B[31m";
pub const GREEN: &str = "\x1B[32m";
pub const YELLOW: &str = "\x1B[33m";
pub const BLUE: &str = "\x1B[34m";
pub const MAGENTA: &str = "\x1B[35m";
pub const CYAN: &str = "\x1b[36m";

pub const HI_BLACK: &str = "\x1B[90m";
pub const HI_RED: &str = "\x1B[91m";
pub const HI_GREEN: &str = "\x1B[92m";
pub const HI_BLUE: &str = "\x1B[94m";
pub const HI_MAGENTA: &str = "\x1B[95m";

pub fn from_colour(num: u32) -> Option<aformat::ArrayString<19>> {
    if num != 0 {
        return Some(aformat!("\x1B[38;2;{};{};{}m", r(num), g(num), b(num)));
    }

    None
}

#[must_use]
pub const fn r(num: u32) -> u8 {
    ((num >> 16) & 255) as u8
}

#[must_use]
pub const fn g(num: u32) -> u8 {
    ((num >> 8) & 255) as u8
}

#[must_use]
pub const fn b(num: u32) -> u8 {
    (num & 255) as u8
}
