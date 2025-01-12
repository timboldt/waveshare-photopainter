use embassy_rp::spi::{self};
use embedded_graphics::{pixelcolor::Rgb888, prelude::*};

mod buffer;
mod driver;
pub use buffer::DisplayBuffer;
pub use driver::EPaper7In3F;

// Display resolution.
pub const EPD_7IN3F_WIDTH: usize = 800;
pub const EPD_7IN3F_HEIGHT: usize = 480;
pub const EPD_7IN3F_IMAGE_SIZE: usize = EPD_7IN3F_WIDTH * EPD_7IN3F_HEIGHT / 2;

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Color {
    Black = 0b000,
    White = 0b001,
    Green = 0b010,
    Blue = 0b011,
    Red = 0b100,
    Yellow = 0b101,
    Orange = 0b110,
    #[allow(dead_code)]
    Clean = 0b111, // Not a real color, used to clear the display.
}
impl Color {
    pub fn from_rgb888(rgb: Rgb888) -> Self {
        match rgb {
            Rgb888::BLACK => Self::Black,
            Rgb888::WHITE => Self::White,
            Rgb888::GREEN => Self::Green,
            Rgb888::BLUE => Self::Blue,
            Rgb888::RED => Self::Red,
            Rgb888::YELLOW => Self::Yellow,
            Rgb888::CSS_ORANGE => Self::Orange,
            _ => Self::White,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Error {
    #[allow(dead_code)]
    Timeout,
    SpiError(spi::Error),
}
