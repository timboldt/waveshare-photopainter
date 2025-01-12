use embassy_rp::{
    gpio,
    spi::{self, Async},
    watchdog::*,
};
use embassy_time::Timer;

use super::{buffer::DisplayBuffer, Color, Error, EPD_7IN3F_HEIGHT, EPD_7IN3F_WIDTH};

pub struct EPaper7In3F<SPI: embassy_rp::spi::Instance + 'static> {
    spi: spi::Spi<'static, SPI, Async>,
    reset_pin: gpio::Output<'static>,
    dc_pin: gpio::Output<'static>,
    cs_pin: gpio::Output<'static>,
    busy_pin: gpio::Input<'static>,
}

impl<SPI> EPaper7In3F<SPI>
where
    SPI: embassy_rp::spi::Instance,
{
    pub fn new(
        spi: spi::Spi<'static, SPI, Async>,
        reset_pin: gpio::Output<'static>,
        dc_pin: gpio::Output<'static>,
        cs_pin: gpio::Output<'static>,
        busy_pin: gpio::Input<'static>,
    ) -> Self {
        EPaper7In3F {
            spi,
            reset_pin,
            dc_pin,
            cs_pin,
            busy_pin,
        }
    }

    /// Initializes the display.
    pub async fn init(&mut self, watchdog: &mut Watchdog) -> Result<(), Error> {
        self.reset().await?;
        self.wait_for_idle(watchdog).await?;
        Timer::after_millis(30).await;

        // Magic initialization sequence: replicated from Waveshare C code.
        self.send_cmd_with_data(0xAA, &[0x49, 0x55, 0x20, 0x08, 0x09, 0x18])
            .await?;
        self.send_cmd_with_data(0x01, &[0x3F, 0x00, 0x32, 0x2A, 0x0E, 0x2A])
            .await?;
        self.send_cmd_with_data(0x00, &[0x5F, 0x69]).await?;
        self.send_cmd_with_data(0x03, &[0x00, 0x54, 0x00, 0x44])
            .await?;
        self.send_cmd_with_data(0x05, &[0x40, 0x1F, 0x1F, 0x2C])
            .await?;
        self.send_cmd_with_data(0x06, &[0x6F, 0x1F, 0x1F, 0x22])
            .await?;
        self.send_cmd_with_data(0x08, &[0x6F, 0x1F, 0x1F, 0x22])
            .await?;
        self.send_cmd_with_data(0x13, &[0x00, 0x04]).await?;
        self.send_cmd_with_data(0x30, &[0x3C]).await?;
        self.send_cmd_with_data(0x41, &[0x00]).await?;
        self.send_cmd_with_data(0x50, &[0x3F]).await?;
        self.send_cmd_with_data(0x60, &[0x02, 0x00]).await?;
        self.send_cmd_with_data(0x61, &[0x03, 0x20, 0x01, 0xE0])
            .await?;
        self.send_cmd_with_data(0x82, &[0x1E]).await?;
        self.send_cmd_with_data(0x84, &[0x00]).await?;
        self.send_cmd_with_data(0x86, &[0x00]).await?;
        self.send_cmd_with_data(0xE3, &[0x2F]).await?;
        self.send_cmd_with_data(0xE0, &[0x00]).await?;
        self.send_cmd_with_data(0xE6, &[0x00]).await?;
        Ok(())
    }

    /// Clears the display with the given color.
    #[allow(dead_code)]
    pub async fn clear(&mut self, color: Color, watchdog: &mut Watchdog) -> Result<(), Error> {
        self.send_cmd(0x10).await?;
        let color = color as u8;
        let data = [color << 4 | color; EPD_7IN3F_WIDTH / 2];
        for _ in 0..EPD_7IN3F_HEIGHT {
            self.send_data(&data).await?;
        }

        self.display_frame(watchdog).await?;
        Ok(())
    }

    /// Draw the seven color blocks on the screen.
    #[allow(dead_code)]
    pub async fn show_seven_color_blocks(&mut self, watchdog: &mut Watchdog) -> Result<(), Error> {
        self.send_cmd(0x10).await?;

        let color_list = [
            Color::White,
            Color::Black,
            Color::Blue,
            Color::Green,
            Color::Orange,
            Color::Red,
            Color::Yellow,
            Color::White,
        ];
        for color in color_list.iter() {
            let color = *color as u8;
            // This consumes 400 bytes of stack memory, which is probably okay?
            // The alternative is to call send_data() 400 times, which is also toggles the GPIOs 400
            // times.
            let data = [color << 4 | color; EPD_7IN3F_WIDTH / 2];
            for _ in 0..EPD_7IN3F_HEIGHT / color_list.len() {
                self.send_data(&data).await?;
            }
        }
        self.display_frame(watchdog).await?;
        Ok(())
    }

    /// Sends the given image to the display.
    pub async fn show_image(
        &mut self,
        image: &DisplayBuffer,
        watchdog: &mut Watchdog,
    ) -> Result<(), Error> {
        self.send_cmd(0x10).await?;

        for y in 0..EPD_7IN3F_HEIGHT {
            let offset = y * EPD_7IN3F_WIDTH / 2;
            self.send_data(&image.frame_buffer[offset..(offset + EPD_7IN3F_WIDTH / 2)])
                .await?;
            watchdog.feed();
        }

        self.display_frame(watchdog).await?;
        Ok(())
    }

    /// Puts the display in deep sleep mode.
    pub async fn deep_sleep(&mut self) -> Result<(), Error> {
        self.send_cmd_with_data(0x07, &[0xA5]).await?;
        Ok(())
    }

    /// Resets the display.
    async fn reset(&mut self) -> Result<(), Error> {
        self.reset_pin.set_high();
        Timer::after_millis(20).await;
        self.reset_pin.set_low();
        Timer::after_millis(5).await;
        self.reset_pin.set_high();
        Timer::after_millis(20).await;
        Ok(())
    }

    /// Sends a command to the display.
    async fn send_cmd(&mut self, command: u8) -> Result<(), Error> {
        // DC low: next byte is command.
        self.dc_pin.set_low();
        // CS low: start command transmission.
        self.cs_pin.set_low();
        // Send the command.
        self.spi.write(&[command]).await.map_err(Error::SpiError)?;
        // CS high: end command transmission.
        self.cs_pin.set_high();
        Ok(())
    }

    // Sends data to the display.
    async fn send_data(&mut self, data: &[u8]) -> Result<(), Error> {
        // DC high: next byte is data.
        self.dc_pin.set_high();
        // CS low: start data transmission.
        self.cs_pin.set_low();
        self.spi.write(data).await.map_err(Error::SpiError)?;
        // CS high: end data transmission.
        self.cs_pin.set_high();
        Ok(())
    }

    /// Sends a command with data, to the display.
    async fn send_cmd_with_data(&mut self, command: u8, data: &[u8]) -> Result<(), Error> {
        self.send_cmd(command).await?;
        self.send_data(data).await?;
        Ok(())
    }

    /// Waits for the display to become idle.
    async fn wait_for_idle(&mut self, watchdog: &mut Watchdog) -> Result<(), Error> {
        let max_delay_ms = 50_000;
        let polling_ms = 10;

        let mut accum_ms = 0;
        while self.busy_pin.is_low() {
            Timer::after_millis(polling_ms).await;
            watchdog.feed();
            accum_ms += polling_ms;
            if accum_ms >= max_delay_ms {
                return Err(Error::Timeout);
            }
        }
        Ok(())
    }

    /// Powers on the display, refreshes (transfers the frame buffer to) the display, and then
    /// powers off the display.
    async fn display_frame(&mut self, watchdog: &mut Watchdog) -> Result<(), Error> {
        // Power on the display.
        self.send_cmd(0x04).await?;
        self.wait_for_idle(watchdog).await?;

        // Refresh the display.
        self.send_cmd_with_data(0x12, &[0x00]).await?;
        self.wait_for_idle(watchdog).await?;

        // Power off the display.
        self.send_cmd_with_data(0x02, &[0x00]).await?;
        self.wait_for_idle(watchdog).await?;

        Ok(())
    }
}
