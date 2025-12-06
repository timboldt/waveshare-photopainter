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
use graphics::draw_random_walk_art;
use panic_probe as _;

mod epaper;
mod graphics;
mod rtc;
mod usb_console;

// Minimum power is 3.1V.
const MIN_BATTERY_MILLIVOLTS: u32 = 3100;

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => AdcInterruptHandler;
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
});

/// Run a single display update cycle
pub async fn run_display_update(
    epaper: &mut epaper::EPaper7In3F<embassy_rp::peripherals::SPI1>,
    watchdog: &mut Watchdog,
    rng: &mut RoscRng,
    activity_led: &mut Output<'_>,
) -> Result<(), ()> {
    info!("Running display update");
    activity_led.set_high();

    epaper.init(watchdog).await.map_err(|_| ())?;
    let display_buf = epaper::DisplayBuffer::get();
    draw_random_walk_art(display_buf, rng.next_u64()).map_err(|_| ())?;
    epaper
        .show_image(display_buf, watchdog)
        .await
        .map_err(|_| ())?;
    epaper.deep_sleep().await.map_err(|_| ())?;

    activity_led.set_low();
    info!("Display update complete");
    Ok(())
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize Peripherals
    let p = embassy_rp::init(Default::default());

    let mut rng = RoscRng;

    // Activity LED: red.
    let mut activity_led_pin = Output::new(p.PIN_25, Level::Low);
    // Power LED: green.
    let mut power_led_pin = Output::new(p.PIN_26, Level::High);
    // User button (low is button pressed, or the auto-switch is enabled).
    let user_button_pin = Input::new(p.PIN_19, Pull::Up);
    // Battery power control (high is enabled; low turns off the power).
    let mut battery_enable_pin = Output::new(p.PIN_18, Level::High);
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
    epd_config.frequency = 8_000_000;
    let epd_spi = Spi::new_txonly(p.SPI1, epd_clk, epd_mosi, p.DMA_CH0, epd_config);

    let epd_reset_pin = Output::new(p.PIN_12, Level::Low);
    let epd_dc_pin = Output::new(p.PIN_8, Level::Low);
    let epd_cs_pin = Output::new(p.PIN_9, Level::High);
    let epd_busy_pin = Input::new(p.PIN_13, Pull::None);
    let mut epd_enable_pin = Output::new(p.PIN_16, Level::High);

    let mut epaper =
        epaper::EPaper7In3F::new(epd_spi, epd_reset_pin, epd_dc_pin, epd_cs_pin, epd_busy_pin);

    // Setup Real Time Clock
    let rtc_sda = p.PIN_14;
    let rtc_scl = p.PIN_15;
    let _rtc_int = p.PIN_6;
    let i2c = i2c::I2c::new_async(p.I2C1, rtc_scl, rtc_sda, Irqs, i2c::Config::default());
    let mut rtc = rtc::Pcf85063::new(i2c);

    // Setup VBAT ADC on pin 29
    let mut adc = Adc::new(p.ADC, Irqs, adc::Config::default());
    let mut v_sys = Channel::new_pin(p.PIN_29, Pull::None);

    // Create a function to read the VSYS ADC value and convert to voltage.
    let mut battery_voltage = || {
        let v = adc.blocking_read(&mut v_sys).unwrap();
        // 3.3V (3300mV) reference voltage, 3x voltage divider, 12-bit ADC (4096).
        v as u32 * 3300 * 3 / 4096
    };
    info!("Battery voltage: {}", battery_voltage());

    // Enable the watchdog timer, in case something goes wrong.
    let mut watchdog = Watchdog::new(p.WATCHDOG);
    watchdog.start(Duration::from_secs(8));

    Timer::after_millis(1000).await;

    rtc.init().await.unwrap();

    info!("Init done");

    // Check if USB power is connected - if so, enter console mode
    if vbus_state_pin.is_high() {
        info!("USB power detected - entering console mode");
        run_usb_console_mode(
            p.USB,
            &mut epaper,
            &mut watchdog,
            &mut rng,
            &mut activity_led_pin,
        )
        .await;
    } else {
        info!("Running on battery - entering normal mode");
        run_normal_mode(
            &mut epaper,
            &mut watchdog,
            &mut rng,
            &mut activity_led_pin,
            &mut power_led_pin,
            &user_button_pin,
            &charge_state_pin,
            &vbus_state_pin,
            &mut battery_voltage,
        )
        .await;
    }

    // Power down
    epd_enable_pin.set_low();
    battery_enable_pin.set_low();

    loop {
        Timer::after(Duration::from_millis(1000)).await;
    }
}

/// USB Console mode - wait for commands over serial
async fn run_usb_console_mode<'d>(
    usb: embassy_rp::Peri<'d, USB>,
    epaper: &mut epaper::EPaper7In3F<embassy_rp::peripherals::SPI1>,
    watchdog: &mut Watchdog,
    rng: &mut RoscRng,
    activity_led: &mut Output<'_>,
) -> ! {
    let mut console = usb_console::UsbConsole::new();

    // This function never returns
    console.run(usb, epaper, watchdog, rng, activity_led).await
}

/// Normal mode - run display on button press or initially
#[allow(clippy::too_many_arguments)]
async fn run_normal_mode(
    epaper: &mut epaper::EPaper7In3F<embassy_rp::peripherals::SPI1>,
    watchdog: &mut Watchdog,
    rng: &mut RoscRng,
    activity_led: &mut Output<'_>,
    power_led: &mut Output<'_>,
    user_button: &Input<'_>,
    charge_state: &Input<'_>,
    vbus_state: &Input<'_>,
    battery_voltage: &mut impl FnMut() -> u32,
) {
    let mut show_display = true;
    let mut button_press_count = 0;

    'main: loop {
        let running_on_battery = vbus_state.is_low();
        info!("Running on battery? {}", running_on_battery);

        // If the battery is low, flash the low power LED, disable the alarm, and turn off the
        // power.
        if running_on_battery && battery_voltage() < MIN_BATTERY_MILLIVOLTS {
            info!("Battery is low");
            for _ in 0..5 {
                power_led.set_high();
                Timer::after(Duration::from_millis(200)).await;
                power_led.set_low();
                Timer::after(Duration::from_millis(100)).await;
            }
            // Exit and power down.
            break 'main;
        }

        // Run the display.
        if show_display {
            let _ = run_display_update(epaper, watchdog, rng, activity_led).await;
            show_display = false;
        }

        if running_on_battery {
            break 'main;
        }

        if charge_state.is_low() {
            // Charging.
            power_led.set_high();
        } else {
            // Not charging.
            power_led.set_low();
        }

        if user_button.is_low() {
            button_press_count += 1;
            if button_press_count > 3 {
                show_display = true;
                button_press_count = 0;
            }
        } else {
            button_press_count = 0;
        }

        watchdog.feed();
        Timer::after(Duration::from_millis(200)).await;
    }
}
