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
            // Feed watchdog while waiting for input
            ctx.watchdog.feed();

            let mut buf = [0u8; 1];

            // Use select to either read a packet or timeout to feed watchdog
            // This prevents watchdog timeout during console inactivity
            let c = loop {
                match embassy_futures::select::select(
                    class.read_packet(&mut buf),
                    Timer::after(Duration::from_secs(4)),
                )
                .await
                {
                    embassy_futures::select::Either::First(result) => {
                        result?;
                        break buf[0];
                    }
                    embassy_futures::select::Either::Second(_) => {
                        // Timeout - feed watchdog and try again
                        ctx.watchdog.feed();
                        continue;
                    }
                }
            };

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
            ConsoleCommand::DrawRandom => {
                self.write_line(class, "Drawing random walk art...").await?;
                match crate::run_display_random(ctx).await {
                    Ok(()) => {
                        self.write_line(class, "Random walk art complete!").await?;
                    }
                    Err(()) => {
                        self.write_line(class, "ERROR: Display update failed")
                            .await?;
                    }
                }
            }
            ConsoleCommand::DrawCalendar => {
                self.write_line(class, "Drawing calendar page...").await?;
                match crate::run_display_calendar(ctx).await {
                    Ok(()) => {
                        self.write_line(class, "Calendar page complete!").await?;
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
            ConsoleCommand::Time => match ctx.rtc.get_time().await {
                Ok(time) => {
                    let mut buf = [0u8; 64];
                    let msg = format_no_std::show(
                        &mut buf,
                        format_args!(
                            "Time: {:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                            time.years,
                            time.months,
                            time.days,
                            time.hours,
                            time.minutes,
                            time.seconds
                        ),
                    )
                    .unwrap_or("Time read error");
                    self.write_line(class, msg).await?;
                }
                Err(_) => {
                    self.write_line(class, "ERROR: Failed to read RTC time")
                        .await?;
                }
            },
            ConsoleCommand::SetTime(time) => {
                // Validate time data
                if time.years < 2000 || time.years > 2099 {
                    self.write_line(class, "ERROR: Year must be 2000-2099")
                        .await?;
                    return Ok(());
                }
                if time.months == 0 || time.months > 12 {
                    self.write_line(class, "ERROR: Month must be 1-12").await?;
                    return Ok(());
                }
                if time.days == 0 || time.days > 31 {
                    self.write_line(class, "ERROR: Day must be 1-31").await?;
                    return Ok(());
                }
                if time.hours > 23 {
                    self.write_line(class, "ERROR: Hour must be 0-23").await?;
                    return Ok(());
                }
                if time.minutes > 59 {
                    self.write_line(class, "ERROR: Minute must be 0-59").await?;
                    return Ok(());
                }
                if time.seconds > 59 {
                    self.write_line(class, "ERROR: Second must be 0-59").await?;
                    return Ok(());
                }

                match ctx.rtc.set_time(&time).await {
                    Ok(()) => {
                        let mut buf = [0u8; 64];
                        let msg = format_no_std::show(
                            &mut buf,
                            format_args!(
                                "Time set to: {:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                                time.years,
                                time.months,
                                time.days,
                                time.hours,
                                time.minutes,
                                time.seconds
                            ),
                        )
                        .unwrap_or("Time set");
                        self.write_line(class, msg).await?;
                    }
                    Err(_) => {
                        self.write_line(class, "ERROR: Failed to set RTC time")
                            .await?;
                    }
                }
            }
            ConsoleCommand::Battery => {
                let voltage = ctx.battery_voltage();
                let charging = ctx.charge_state.is_low();
                let usb_power = ctx.vbus_state.is_high();

                let mut buf = [0u8; 128];
                let msg = format_no_std::show(&mut buf, format_args!("Battery: {}mV", voltage))
                    .unwrap_or("Battery voltage");
                self.write_line(class, msg).await?;

                let status_msg = match (usb_power, charging) {
                    (true, true) => "USB power connected, charging",
                    (true, false) => "USB power connected, not charging (full)",
                    (false, _) => "Running on battery",
                };
                self.write_line(class, status_msg).await?;

                // Show battery status
                if !usb_power && voltage < crate::MIN_BATTERY_MILLIVOLTS {
                    self.write_line(class, "WARNING: Battery voltage is low!")
                        .await?;
                }
            }
            ConsoleCommand::Reset => {
                self.write_line(class, "Resetting device...").await?;

                // Small delay to ensure message is sent
                Timer::after(Duration::from_millis(100)).await;

                info!("System reset requested");

                // Perform system reset
                cortex_m::peripheral::SCB::sys_reset();
            }
            ConsoleCommand::Version => {
                let version = env!("CARGO_PKG_VERSION");
                let build_date = env!("BUILD_DATE");
                let mut buf = [0u8; 128];
                let msg = format_no_std::show(
                    &mut buf,
                    format_args!("Firmware version: {} (built {})", version, build_date),
                )
                .unwrap_or("Firmware version");
                self.write_line(class, msg).await?;
            }
            ConsoleCommand::Clear => {
                self.write_line(class, "Clearing display to white...")
                    .await?;

                // Get display buffer and clear it (0x11 = White color, two pixels per byte)
                let display_buf = crate::epaper::DisplayBuffer::get();
                display_buf.frame_buffer.fill(0x11);

                // Initialize and show the cleared display
                match ctx.epaper.init(&mut ctx.watchdog).await {
                    Ok(()) => match ctx.epaper.show_image(display_buf, &mut ctx.watchdog).await {
                        Ok(()) => {
                            let _ = ctx.epaper.deep_sleep().await;
                            self.write_line(class, "Display cleared!").await?;
                        }
                        Err(_) => {
                            self.write_line(class, "ERROR: Failed to update display")
                                .await?;
                        }
                    },
                    Err(_) => {
                        self.write_line(class, "ERROR: Failed to initialize display")
                            .await?;
                    }
                }
            }
            ConsoleCommand::Dfu => {
                self.write_line(class, "Rebooting to USB bootloader (UF2 mode)...")
                    .await?;

                // Small delay to ensure message is sent
                Timer::after(Duration::from_millis(100)).await;

                info!("Resetting to USB boot mode");

                // Disable interrupts to ensure clean state
                cortex_m::interrupt::disable();

                // Reset to USB bootloader mode with clean state
                // The ROM function handles setting up the watchdog scratch registers
                // and triggering a reset that enters the bootloader
                embassy_rp::rom_data::reset_to_usb_boot(0, 0);

                // This line should never be reached
                loop {
                    cortex_m::asm::wfi();
                }
            }
            ConsoleCommand::Help => {
                self.write_line(class, "Available commands:").await?;
                self.write_line(class, "Display Commands:").await?;
                self.write_line(
                    class,
                    "  DRAWCALENDAR - Draw calendar page with date and quote",
                )
                .await?;
                self.write_line(class, "  DRAWRANDOM   - Draw random walk art")
                    .await?;
                self.write_line(class, "  GO           - Alias for DRAWCALENDAR")
                    .await?;
                self.write_line(class, "  CLEAR        - Clear display to white")
                    .await?;
                self.write_line(class, "").await?;
                self.write_line(class, "RTC Commands:").await?;
                self.write_line(class, "  TIME         - Display current RTC time")
                    .await?;
                self.write_line(class, "  SETTIME Y M D H M S - Set RTC time")
                    .await?;
                self.write_line(
                    class,
                    "                 Example: SETTIME 2025 12 6 14 39 30",
                )
                .await?;
                self.write_line(
                    class,
                    "  SLEEP n      - Deep sleep for n seconds, RTC alarm wakes",
                )
                .await?;
                self.write_line(class, "").await?;
                self.write_line(class, "System Commands:").await?;
                self.write_line(class, "  BATTERY      - Show battery voltage and status")
                    .await?;
                self.write_line(class, "  VERSION      - Show firmware version")
                    .await?;
                self.write_line(class, "  RESET        - Soft reset device")
                    .await?;
                self.write_line(
                    class,
                    "  DFU          - Reboot to USB bootloader (UF2 mode)",
                )
                .await?;
                self.write_line(class, "  HELP or ?    - Show this help")
                    .await?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConsoleCommand {
    DrawRandom,
    DrawCalendar,
    Sleep(u32),
    Time,
    SetTime(crate::rtc::TimeData),
    Battery,
    Reset,
    Version,
    Clear,
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

    let parts: heapless::Vec<&str, 8> = upper.split_whitespace().collect();

    if parts.is_empty() {
        return None;
    }

    match parts[0] {
        "DRAWRANDOM" => Some(ConsoleCommand::DrawRandom),
        "DRAWCALENDAR" | "GO" => Some(ConsoleCommand::DrawCalendar), /* GO is alias for */
        // DRAWCALENDAR
        "SLEEP" => {
            if parts.len() < 2 {
                None
            } else {
                parts[1].parse::<u32>().ok().map(ConsoleCommand::Sleep)
            }
        }
        "TIME" => Some(ConsoleCommand::Time),
        "SETTIME" => {
            // SETTIME expects 6 arguments: year month day hour minute second
            if parts.len() < 7 {
                None
            } else {
                let year = parts[1].parse::<u16>().ok()?;
                let month = parts[2].parse::<u16>().ok()?;
                let day = parts[3].parse::<u16>().ok()?;
                let hour = parts[4].parse::<u16>().ok()?;
                let minute = parts[5].parse::<u16>().ok()?;
                let second = parts[6].parse::<u16>().ok()?;

                Some(ConsoleCommand::SetTime(crate::rtc::TimeData {
                    years: year,
                    months: month,
                    days: day,
                    hours: hour,
                    minutes: minute,
                    seconds: second,
                }))
            }
        }
        "BATTERY" | "VBAT" => Some(ConsoleCommand::Battery),
        "RESET" => Some(ConsoleCommand::Reset),
        "VERSION" | "VER" => Some(ConsoleCommand::Version),
        "CLEAR" => Some(ConsoleCommand::Clear),
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
