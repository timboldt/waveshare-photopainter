use embedded_graphics::{pixelcolor::Rgb888, prelude::*, primitives::Rectangle};

use super::{Color, Error, EPD_7IN3F_HEIGHT, EPD_7IN3F_IMAGE_SIZE, EPD_7IN3F_WIDTH};

pub struct DisplayBuffer {
    pub frame_buffer: [u8; EPD_7IN3F_IMAGE_SIZE],
}

// The display buffer is a singleton, because of the large memory requirements.
static mut DISPLAY_BUF: DisplayBuffer = DisplayBuffer {
    frame_buffer: [0; EPD_7IN3F_IMAGE_SIZE],
};

impl DisplayBuffer {
    /// Returns a mutable reference to the one and only display buffer.
    /// The mutable static is necessary because a single display buffer needs about 80% of all the
    /// RAM on a Pico.
    pub fn get() -> &'static mut Self {
        unsafe { &mut *core::ptr::addr_of_mut!(DISPLAY_BUF) }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        let index = (x + y * EPD_7IN3F_WIDTH) / 2;
        let color = color as u8;
        if x % 2 == 0 {
            self.frame_buffer[index] = (self.frame_buffer[index] & 0xF0) | (color & 0x0F);
        } else {
            self.frame_buffer[index] = ((color << 4) & 0xF0) | (self.frame_buffer[index] & 0x0F);
        }
    }
}

impl Dimensions for DisplayBuffer {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(
            Point::new(0, 0),
            Size::new(EPD_7IN3F_WIDTH as u32, EPD_7IN3F_HEIGHT as u32),
        )
    }
}

impl DrawTarget for DisplayBuffer {
    type Color = Rgb888;
    type Error = Error;

    // fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
    //     if area.is_zero_sized() {
    //         return Ok(());
    //     }

    //     let color = Color::from_rgb888(color);
    //     for y in area.top_left.y as usize..area.bottom_right().unwrap().y as usize {
    //         for x in area.top_left.x as usize..area.bottom_right().unwrap().x as usize {
    //             DisplayBuffer::set_pixel(self, x, y, color);
    //         }
    //     }
    //     Ok(())
    // }

    // fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    // where
    //     I: IntoIterator<Item = Self::Color>,
    // {
    //     let mut colors = colors.into_iter();
    //     for y in area.top as usize..area.bottom as usize {
    //         for x in area.left as usize..area.right as usize {
    //             let color = Color::from_rgb888(colors.next().unwrap());
    //             self.set_pixel(x, y, color);
    //         }
    //     }
    // }

    // fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
    //     let color = Color::from_rgb888(color);
    //     for y in 0..EPD_7IN3F_HEIGHT {
    //         for x in 0..EPD_7IN3F_WIDTH {
    //             self.set_pixel(x, y, color);
    //         }
    //     }
    //     Ok(())
    // }

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if coord.x < 0
                || coord.y < 0
                || coord.x >= EPD_7IN3F_WIDTH as i32
                || coord.y >= EPD_7IN3F_HEIGHT as i32
            {
                continue;
            }
            let (x, y) = (coord.x as usize, coord.y as usize);
            DisplayBuffer::set_pixel(self, x, y, Color::from_rgb888(color));
        }

        Ok(())
    }
}
