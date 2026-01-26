use embedded_graphics::{pixelcolor::Rgb888, prelude::*, primitives::Rectangle};

use super::{Color, Error, EPD_7IN3F_HEIGHT, EPD_7IN3F_IMAGE_SIZE, EPD_7IN3F_WIDTH};

pub struct DisplayBuffer {
    pub frame_buffer: [u8; EPD_7IN3F_IMAGE_SIZE],
    pub rotate_180: bool,
}

// The display buffer is a singleton, because of the large memory requirements.
static mut DISPLAY_BUF: DisplayBuffer = DisplayBuffer {
    frame_buffer: [0x11; EPD_7IN3F_IMAGE_SIZE],
    rotate_180: true, // Set to true for 180-degree rotation
};

impl DisplayBuffer {
    /// Returns a mutable reference to the one and only display buffer.
    /// The mutable static is necessary because a single display buffer needs about 80% of all the
    /// RAM on a Pico.
    ///
    /// # Safety
    /// This is safe in the single-threaded embedded context.
    /// The RP2040 runs single-threaded with no preemption in this application.
    pub fn get() -> &'static mut Self {
        unsafe { &mut *core::ptr::addr_of_mut!(DISPLAY_BUF) }
    }

    /// Apply 180-degree rotation to coordinates if enabled
    #[inline]
    fn apply_rotation_usize(&self, x: usize, y: usize) -> (usize, usize) {
        if self.rotate_180 {
            (EPD_7IN3F_WIDTH - 1 - x, EPD_7IN3F_HEIGHT - 1 - y)
        } else {
            (x, y)
        }
    }

    /// Apply 180-degree rotation to i32 coordinates if enabled
    #[inline]
    fn apply_rotation_i32(&self, x: i32, y: i32) -> (i32, i32) {
        if self.rotate_180 {
            (
                EPD_7IN3F_WIDTH as i32 - 1 - x,
                EPD_7IN3F_HEIGHT as i32 - 1 - y,
            )
        } else {
            (x, y)
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= EPD_7IN3F_WIDTH || y >= EPD_7IN3F_HEIGHT {
            return;
        }

        // Apply 180-degree rotation if enabled
        let (x, y) = self.apply_rotation_usize(x, y);

        let index = (x + y * EPD_7IN3F_WIDTH) / 2;
        let color = color as u8;
        if x.is_multiple_of(2) {
            self.frame_buffer[index] = ((color << 4) & 0xF0) | (self.frame_buffer[index] & 0x0F);
        } else {
            self.frame_buffer[index] = (self.frame_buffer[index] & 0xF0) | (color & 0x0F);
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

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            // Apply rotation before bounds checking
            let (x, y) = self.apply_rotation_i32(coord.x, coord.y);

            if x < 0 || y < 0 || x >= EPD_7IN3F_WIDTH as i32 || y >= EPD_7IN3F_HEIGHT as i32 {
                continue;
            }

            let index = (x as usize + y as usize * EPD_7IN3F_WIDTH) / 2;
            let color_val = Color::from_rgb888(color) as u8;
            if (x as usize).is_multiple_of(2) {
                self.frame_buffer[index] =
                    ((color_val << 4) & 0xF0) | (self.frame_buffer[index] & 0x0F);
            } else {
                self.frame_buffer[index] = (self.frame_buffer[index] & 0xF0) | (color_val & 0x0F);
            }
        }

        Ok(())
    }
}
