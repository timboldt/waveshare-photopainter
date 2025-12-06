use defmt::*;
use embassy_rp::{
    bind_interrupts,
    peripherals::USB,
    usb::{Driver, InterruptHandler},
};
use embassy_time::{Duration, Timer};
use embassy_usb::{
    class::cdc_acm::{CdcAcmClass, State},
    driver::EndpointError,
    Builder, Config,
};

bind_interrupts!(pub struct UsbIrqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

const MAX_PACKET_SIZE: u8 = 64;
const READ_BUF_SIZE: usize = 128;

pub struct UsbConsole {
    read_buf: [u8; READ_BUF_SIZE],
    read_pos: usize,
}

impl UsbConsole {
    pub fn new() -> Self {
        Self {
            read_buf: [0u8; READ_BUF_SIZE],
            read_pos: 0,
        }
    }

    /// Initialize and run the USB console
    pub async fn run<'d>(
        &mut self,
        usb: embassy_rp::Peri<'d, USB>,
        mut ctx: crate::DeviceContext,
    ) -> ! {
        info!("Starting USB console");

        // Create the USB driver
        let driver = Driver::new(usb, UsbIrqs);

        // Create embassy-usb Config
        let mut config = Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("Waveshare");
        config.product = Some("Photo Painter Console");
        config.serial_number = Some("12345678");
        config.max_power = 100;
        config.max_packet_size_0 = MAX_PACKET_SIZE;

        // Device and config descriptor buffer
        let mut device_descriptor = [0; 256];
        let mut config_descriptor = [0; 256];
        let mut bos_descriptor = [0; 256];
        let mut control_buf = [0; 64];

        let mut state = State::new();

        // Create embassy-usb DeviceBuilder
        let mut builder = Builder::new(
            driver,
            config,
            &mut device_descriptor,
            &mut config_descriptor,
            &mut bos_descriptor,
            &mut control_buf,
        );

        // Create CDC-ACM class (USB serial)
        let mut class = CdcAcmClass::new(&mut builder, &mut state, MAX_PACKET_SIZE as u16);

        // Build the USB device
        let mut usb_dev = builder.build();

        // Run the USB device in a separate task
        let usb_fut = usb_dev.run();

        // Run the console
        let console_fut = async {
            loop {
                class.wait_connection().await;
                info!("USB connected");
                let _ = self.write_line(&mut class, "").await;
                let _ = self
                    .write_line(&mut class, "Waveshare Photo Painter Console")
                    .await;
                let _ = self.write_line(&mut class, "Type HELP for commands").await;
                let _ = self.write_prompt(&mut class).await;

                // Main console loop
                let _ = self.run_console(&mut class, &mut ctx).await;

                info!("USB disconnected");
            }
        };

        // Run both futures concurrently
        embassy_futures::join::join(usb_fut, console_fut).await;

        core::unreachable!()
    }

    async fn run_console<'d>(
        &mut self,
        class: &mut CdcAcmClass<'d, Driver<'d, USB>>,
        ctx: &mut crate::DeviceContext,
    ) -> Result<(), EndpointError> {
        loop {
            let mut buf = [0u8; 1];
            class.read_packet(&mut buf).await?;
            let c = buf[0];

            // Echo the character
            class.write_packet(&[c]).await?;

            // Handle special characters
            match c {
                b'\r' | b'\n' => {
                    // End of line - process command
                    class.write_packet(b"\r\n").await?;

                    if self.read_pos > 0 {
                        let cmd_str =
                            core::str::from_utf8(&self.read_buf[..self.read_pos]).unwrap_or("");
                        info!("Command: {}", cmd_str);

                        if let Some(cmd) = parse_command(cmd_str) {
                            self.execute_command(class, ctx, cmd).await?;
                        } else {
                            self.write_line(class, "Unknown command. Type HELP for help.")
                                .await?;
                        }

                        self.read_pos = 0;
                    }

                    self.write_prompt(class).await?;
                }
                0x08 | 0x7F => {
                    // Backspace or DEL
                    if self.read_pos > 0 {
                        self.read_pos -= 1;
                        // Move cursor back, print space, move back again
                        class.write_packet(b"\x08 \x08").await?;
                    }
                }
                _ if c.is_ascii_graphic() || c == b' ' => {
                    // Printable character
                    if self.read_pos < READ_BUF_SIZE {
                        self.read_buf[self.read_pos] = c;
                        self.read_pos += 1;
                    }
                }
                _ => {
                    // Ignore other characters
                }
            }
        }
    }

    async fn write_prompt<'a>(
        &self,
        class: &mut CdcAcmClass<'a, Driver<'a, USB>>,
    ) -> Result<(), EndpointError> {
        class.write_packet(b"> ").await
    }

    async fn write_line<'a>(
        &self,
        class: &mut CdcAcmClass<'a, Driver<'a, USB>>,
        msg: &str,
    ) -> Result<(), EndpointError> {
        class.write_packet(msg.as_bytes()).await?;
        class.write_packet(b"\r\n").await
    }

    async fn execute_command<'a>(
        &mut self,
        class: &mut CdcAcmClass<'a, Driver<'a, USB>>,
        ctx: &mut crate::DeviceContext,
        cmd: ConsoleCommand,
    ) -> Result<(), EndpointError> {
        match cmd {
            ConsoleCommand::Go => {
                self.write_line(class, "Running display update...").await?;
                match crate::run_display_update(ctx).await {
                    Ok(()) => {
                        self.write_line(class, "Display update complete!").await?;
                    }
                    Err(()) => {
                        self.write_line(class, "ERROR: Display update failed")
                            .await?;
                    }
                }
            }
            ConsoleCommand::Sleep(seconds) => {
                if seconds == 0 {
                    self.write_line(class, "ERROR: Sleep time must be > 0")
                        .await?;
                    return Ok(());
                }

                let mut buf = [0u8; 64];
                let msg = format_no_std::show(
                    &mut buf,
                    format_args!("Deep sleep for {} seconds (power off)...", seconds),
                )
                .unwrap_or("Deep sleep...");
                self.write_line(class, msg).await?;

                // Get current RTC time
                let current_time = match ctx.rtc.get_time().await {
                    Ok(t) => t,
                    Err(_) => {
                        self.write_line(class, "ERROR: Failed to read RTC time")
                            .await?;
                        return Ok(());
                    }
                };

                // Calculate alarm time
                let alarm_time = crate::rtc::add_seconds_to_time(&current_time, seconds);

                // Set current time (in case RTC drifted)
                if ctx.rtc.set_time(&current_time).await.is_err() {
                    self.write_line(class, "ERROR: Failed to set RTC time")
                        .await?;
                    return Ok(());
                }

                // Set RTC alarm
                if ctx.rtc.set_alarm(&alarm_time).await.is_err() {
                    self.write_line(class, "ERROR: Failed to set RTC alarm")
                        .await?;
                    return Ok(());
                }

                // Clear any existing alarm flag
                let _ = ctx.rtc.clear_alarm_flag().await;

                // Small delay to ensure message is sent
                Timer::after(Duration::from_millis(100)).await;

                info!("Powering off - RTC alarm will wake in {} seconds", seconds);

                // Turn off E-paper display power
                ctx.epd_enable.set_low();

                // Power off the device by disabling battery
                // The RTC INT pin is connected to a MOSFET that will power the device back on
                // when the alarm triggers
                ctx.battery_enable.set_low();

                // This line should never be reached, but just in case...
                loop {
                    Timer::after(Duration::from_secs(1)).await;
                }
            }
            ConsoleCommand::Dfu => {
                self.write_line(class, "Rebooting to USB bootloader (UF2 mode)...")
                    .await?;

                // Small delay to ensure message is sent
                Timer::after(Duration::from_millis(100)).await;

                info!("Resetting to USB boot mode");

                // Reset to USB bootloader mode
                embassy_rp::rom_data::reset_to_usb_boot(0, 0);

                // This line should never be reached
                loop {
                    Timer::after(Duration::from_secs(1)).await;
                }
            }
            ConsoleCommand::Help => {
                self.write_line(class, "Available commands:").await?;
                self.write_line(class, "  GO        - Run display update")
                    .await?;
                self.write_line(class, "  SLEEP n   - Deep sleep (power off) for n seconds")
                    .await?;
                self.write_line(class, "              RTC alarm will power device back on")
                    .await?;
                self.write_line(class, "  DFU       - Reboot to USB bootloader (UF2 mode)")
                    .await?;
                self.write_line(class, "  HELP or ? - Show this help")
                    .await?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConsoleCommand {
    Go,
    Sleep(u32),
    Dfu,
    Help,
}

/// Parse a command string into a ConsoleCommand
pub fn parse_command(cmd: &str) -> Option<ConsoleCommand> {
    let cmd = cmd.trim();

    // Convert to uppercase for case-insensitive matching
    let mut upper = heapless::String::<64>::new();
    for ch in cmd.chars() {
        let _ = upper.push(ch.to_ascii_uppercase());
    }

    let parts: heapless::Vec<&str, 4> = upper.split_whitespace().collect();

    if parts.is_empty() {
        return None;
    }

    match parts[0] {
        "GO" => Some(ConsoleCommand::Go),
        "SLEEP" => {
            if parts.len() < 2 {
                None
            } else {
                parts[1].parse::<u32>().ok().map(ConsoleCommand::Sleep)
            }
        }
        "DFU" => Some(ConsoleCommand::Dfu),
        "HELP" | "?" => Some(ConsoleCommand::Help),
        _ => None,
    }
}

// Simple no_std string formatting helper
mod format_no_std {
    use core::fmt::Write;

    pub struct BufferWriter<'a> {
        buf: &'a mut [u8],
        pos: usize,
    }

    impl<'a> BufferWriter<'a> {
        #[allow(dead_code)]
        pub fn new(buf: &'a mut [u8]) -> Self {
            Self { buf, pos: 0 }
        }
    }

    impl<'a> Write for BufferWriter<'a> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let bytes = s.as_bytes();
            let remaining = self.buf.len() - self.pos;
            if bytes.len() > remaining {
                return Err(core::fmt::Error);
            }
            self.buf[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
            self.pos += bytes.len();
            Ok(())
        }
    }

    pub fn show<'a>(
        buf: &'a mut [u8],
        args: core::fmt::Arguments,
    ) -> Result<&'a str, core::fmt::Error> {
        let pos;
        {
            let mut writer = BufferWriter { buf, pos: 0 };
            write!(writer, "{}", args)?;
            pos = writer.pos;
        }
        Ok(core::str::from_utf8(&buf[..pos]).unwrap_or(""))
    }
}
