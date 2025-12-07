#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::{
    adc::{self, Adc, Channel, InterruptHandler as AdcInterruptHandler},
    bind_interrupts,
    clocks::RoscRng,
    gpio,
    i2c::{self},
    peripherals::{I2C1, USB},
    spi::{self, Spi},
    watchdog::*,
};
use embassy_time::{Duration, Timer};
use gpio::{Input, Level, Output, Pull};
use graphics::{draw_calendar_page, draw_random_walk_art};
use panic_probe as _;

mod epaper;
mod graphics;
mod rtc;
mod usb_console;

/// Minimum battery voltage (3.1V) - below this, the device will shut down
pub const MIN_BATTERY_MILLIVOLTS: u32 = 3100;

/// E-paper SPI frequency: 8 MHz (maximum supported by the display)
const EPD_SPI_FREQUENCY: u32 = 8_000_000;

/// Context struct that owns all device peripherals
pub struct DeviceContext {
    pub epaper: epaper::EPaper7In3F<embassy_rp::peripherals::SPI1>,
    pub watchdog: Watchdog,
    pub rng: RoscRng,
    pub activity_led: Output<'static>,
    pub power_led: Output<'static>,
    pub user_button: Input<'static>,
    pub charge_state: Input<'static>,
    pub vbus_state: Input<'static>,
    pub battery_enable: Output<'static>,
    pub epd_enable: Output<'static>,
    pub rtc: rtc::Pcf85063<embassy_rp::peripherals::I2C1>,
    pub rtc_int_pin: Input<'static>,
    pub adc: Adc<'static, adc::Blocking>,
    pub v_sys: Channel<'static>,
}

impl DeviceContext {
    /// Read battery voltage in millivolts
    /// Returns 0 if ADC read fails (should be rare in normal operation)
    pub fn battery_voltage(&mut self) -> u32 {
        match self.adc.blocking_read(&mut self.v_sys) {
            Ok(v) => {
                // 3.3V (3300mV) reference voltage, 3x voltage divider, 12-bit ADC (4096).
                v as u32 * 3300 * 3 / 4096
            }
            Err(_) => {
                // ADC read failed - return 0 to indicate error
                // This should be rare and indicates a hardware issue
                0
            }
        }
    }
}

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => AdcInterruptHandler;
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
});

/// Run display update with calendar page (default mode)
pub async fn run_display_update(ctx: &mut DeviceContext) -> Result<(), ()> {
    run_display_calendar(ctx).await
}

/// Draw random walk art on the display
pub async fn run_display_random(ctx: &mut DeviceContext) -> Result<(), ()> {
    info!("Running random walk art display");
    ctx.activity_led.set_high();

    ctx.epaper.init(&mut ctx.watchdog).await.map_err(|_| ())?;
    let display_buf = epaper::DisplayBuffer::get();

    draw_random_walk_art(display_buf, ctx.rng.next_u64()).map_err(|_| ())?;

    ctx.epaper
        .show_image(display_buf, &mut ctx.watchdog)
        .await
        .map_err(|_| ())?;
    ctx.epaper.deep_sleep().await.map_err(|_| ())?;

    ctx.activity_led.set_low();
    info!("Random walk art display complete");
    Ok(())
}

/// Draw calendar page with date and quote on the display
pub async fn run_display_calendar(ctx: &mut DeviceContext) -> Result<(), ()> {
    info!("Running calendar page display");
    ctx.activity_led.set_high();

    // Get current time from RTC
    let current_time = ctx
        .rtc
        .get_time()
        .await
        .map_err(|_| {
            info!("Failed to read RTC time, using default");
        })
        .unwrap_or(rtc::DEFAULT_TIME);

    ctx.epaper.init(&mut ctx.watchdog).await.map_err(|_| ())?;
    let display_buf = epaper::DisplayBuffer::get();

    // Draw calendar page with current date and quote
    draw_calendar_page(display_buf, &current_time, ctx.rng.next_u64()).map_err(|_| ())?;

    ctx.epaper
        .show_image(display_buf, &mut ctx.watchdog)
        .await
        .map_err(|_| ())?;
    ctx.epaper.deep_sleep().await.map_err(|_| ())?;

    ctx.activity_led.set_low();
    info!("Calendar page display complete");
    Ok(())
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize Peripherals
    let p = embassy_rp::init(Default::default());

    let rng = RoscRng;

    // Activity LED: red.
    let activity_led_pin = Output::new(p.PIN_25, Level::Low);
    // Power LED: green.
    let power_led_pin = Output::new(p.PIN_26, Level::High);
    // User button (low is button pressed, or the auto-switch is enabled).
    let user_button_pin = Input::new(p.PIN_19, Pull::Up);
    // Battery power control (high is enabled; low turns off the power).
    let battery_enable_pin = Output::new(p.PIN_18, Level::High);
    // Battery charging indicator (low is charging; high is not charging).
    let charge_state_pin = Input::new(p.PIN_17, Pull::Up);
    // USB bus power (high means there is power).
    let vbus_state_pin = Input::new(p.PIN_24, Pull::None);
    // Mystery pin 23, aka "Power_Mode".
    let _power_mode_pin = Input::new(p.PIN_23, Pull::None);

    // Set up E-Paper Display
    let epd_clk = p.PIN_10;
    let epd_mosi = p.PIN_11;
    let mut epd_config = spi::Config::default();
    epd_config.frequency = EPD_SPI_FREQUENCY;
    let epd_spi = Spi::new_txonly(p.SPI1, epd_clk, epd_mosi, p.DMA_CH0, epd_config);

    let epd_reset_pin = Output::new(p.PIN_12, Level::Low);
    let epd_dc_pin = Output::new(p.PIN_8, Level::Low);
    let epd_cs_pin = Output::new(p.PIN_9, Level::High);
    let epd_busy_pin = Input::new(p.PIN_13, Pull::None);
    let epd_enable_pin = Output::new(p.PIN_16, Level::High);

    let epaper =
        epaper::EPaper7In3F::new(epd_spi, epd_reset_pin, epd_dc_pin, epd_cs_pin, epd_busy_pin);

    // Setup Real Time Clock
    let rtc_sda = p.PIN_14;
    let rtc_scl = p.PIN_15;
    let rtc_int_pin = Input::new(p.PIN_6, Pull::Up);
    let i2c = i2c::I2c::new_async(p.I2C1, rtc_scl, rtc_sda, Irqs, i2c::Config::default());
    let rtc = rtc::Pcf85063::new(i2c);

    // Setup VBAT ADC on pin 29
    let adc = Adc::new_blocking(p.ADC, adc::Config::default());
    let v_sys = Channel::new_pin(p.PIN_29, Pull::None);

    // Enable the watchdog timer, in case something goes wrong.
    let mut watchdog = Watchdog::new(p.WATCHDOG);
    watchdog.start(Duration::from_secs(8));

    Timer::after_millis(1000).await;

    // Create device context
    let mut ctx = DeviceContext {
        epaper,
        watchdog,
        rng,
        activity_led: activity_led_pin,
        power_led: power_led_pin,
        user_button: user_button_pin,
        charge_state: charge_state_pin,
        vbus_state: vbus_state_pin,
        battery_enable: battery_enable_pin,
        epd_enable: epd_enable_pin,
        rtc,
        rtc_int_pin,
        adc,
        v_sys,
    };

    info!("Battery voltage: {}", ctx.battery_voltage());

    ctx.rtc.init().await.unwrap();

    info!("Init done");

    // Check if USB power is connected - if so, enter console mode
    if ctx.vbus_state.is_high() {
        info!("USB power detected - entering console mode");
        run_usb_console_mode(p.USB, ctx).await;
    } else {
        info!("Running on battery - entering normal mode");
        run_normal_mode(ctx).await;
    }
}

/// USB Console mode - wait for commands over serial
async fn run_usb_console_mode<'d>(usb: embassy_rp::Peri<'d, USB>, ctx: DeviceContext) -> ! {
    let mut console = usb_console::UsbConsole::new();

    // This function never returns
    console.run(usb, ctx).await
}

/// Normal mode - run display on button press or initially
async fn run_normal_mode(mut ctx: DeviceContext) -> ! {
    let running_on_battery = ctx.vbus_state.is_low();
    info!("Running on battery? {}", running_on_battery);

    // If the battery is low, flash the low power LED and power down
    if running_on_battery && ctx.battery_voltage() < MIN_BATTERY_MILLIVOLTS {
        info!("Battery is low");
        for _ in 0..5 {
            ctx.power_led.set_high();
            Timer::after(Duration::from_millis(200)).await;
            ctx.power_led.set_low();
            Timer::after(Duration::from_millis(100)).await;
        }
        // Power down
        ctx.epd_enable.set_low();
        ctx.battery_enable.set_low();
        loop {
            Timer::after(Duration::from_millis(1000)).await;
        }
    }

    // Battery mode: Display calendar, set 6am alarm, and power off
    if running_on_battery {
        info!("Battery mode: displaying calendar and setting 6am alarm");

        // Display the calendar
        let _ = run_display_calendar(&mut ctx).await;

        // Get current time and calculate next 6am
        let current_time = ctx.rtc.get_time().await.unwrap_or(rtc::DEFAULT_TIME);

        let alarm_time = rtc::calculate_next_6am(&current_time);
        info!(
            "Current time: {:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            current_time.years,
            current_time.months,
            current_time.days,
            current_time.hours,
            current_time.minutes,
            current_time.seconds
        );
        info!(
            "Setting alarm for: {:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            alarm_time.years,
            alarm_time.months,
            alarm_time.days,
            alarm_time.hours,
            alarm_time.minutes,
            alarm_time.seconds
        );

        // Set the RTC alarm and clear any existing alarm flag
        let _ = ctx.rtc.clear_alarm_flag().await;
        let _ = ctx.rtc.set_alarm(&alarm_time).await;

        info!("Powering down until 6am");

        // Power down
        ctx.epd_enable.set_low();
        ctx.battery_enable.set_low();

        loop {
            Timer::after(Duration::from_millis(1000)).await;
        }
    }

    // USB mode: Wait for button presses and show display on demand
    info!("USB mode: waiting for button presses");
    let mut show_display = true;
    let mut button_press_count = 0;

    loop {
        if ctx.charge_state.is_low() {
            // Charging.
            ctx.power_led.set_high();
        } else {
            // Not charging.
            ctx.power_led.set_low();
        }

        // Run the display on first boot or button press
        if show_display {
            let _ = run_display_update(&mut ctx).await;
            show_display = false;
        }

        if ctx.user_button.is_low() {
            button_press_count += 1;
            if button_press_count > 3 {
                show_display = true;
                button_press_count = 0;
            }
        } else {
            button_press_count = 0;
        }

        ctx.watchdog.feed();
        Timer::after(Duration::from_millis(200)).await;
    }
}
