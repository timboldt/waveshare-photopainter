#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::adc::{self, Adc, Channel, InterruptHandler};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio;
use embassy_rp::spi::{self, Spi};
use embassy_rp::watchdog::*;
use embassy_time::{Duration, Timer};
use gpio::{Input, Level, Output, Pull};
use {defmt_rtt as _, panic_probe as _};

mod epaper;

// Minimum power is 3.1V.
const MIN_BATTERY_MILLIVOLTS: u32 = 3100;

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

// Original C code behavior:
//
// init:
//   stdio logging
//   epd spi
//   sd spi
//   rtc i2c
//   VBAT ADC on pin 29
//   gpio init:
//     4x epd pins
//     2x led pins
//     3x charge/battery pins
//     2x power control pins
// watchdog enable
// sleep 1s
// rtc init
// rtc set alarm
// enable charge state IRQ callback
// if battery is low:
//   disable alarm
//   flash low power led
//   turn power off
// led on
// read SD card
// if no main power
//    run display
//    turn power off
// in a loop:
//    wait for key press
//    run display

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize Peripherals
    let p = embassy_rp::init(Default::default());

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

    // If USB connected, set up logging over USB.
    if vbus_state_pin.is_high() {
        info!("USB power detected.");
        // TODO(tboldt): Set up logging over USB.
    }

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

    // TODO(tboldt): Setup SD card SPI
    // #define SD_CS_PIN       5
    // #define SD_CLK_PIN      2
    // #define SD_MOSI_PIN     3
    // #define SD_MISO_PIN     4

    // TODO(tboldt): Setup Real Time Clock
    // #define RTC_SDA         14
    // #define RTC_SCL         15
    // #define RTC_INT         6

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
    //xxx watchdog.start(Duration::from_secs(8));

    Timer::after_millis(1000).await;

    // TODO(tboldt): rtc init
    // TODO(tboldt): rtc set alarm
    // TODO(tboldt): enable charge state IRQ callback

    // TODO(tboldt): main loop
    // if battery is low:
    //   disable alarm
    //   flash low power led
    //   turn power off
    // led on
    // read SD card
    // if no main power
    //    run display
    //    turn power off
    // in a loop:
    //    wait for key press
    //    run display

    info!("Init done");

    let mut show_display = true;
    let mut button_press_count = 0;
    'main: loop {
        let running_on_battery = vbus_state_pin.is_low();
        info!("Running on battery? {}", running_on_battery);

        // If the battery is low, flash the low power LED, disable the alarm, and turn off the power.
        if running_on_battery && battery_voltage() < MIN_BATTERY_MILLIVOLTS {
            info!("Battery is low");
            // TODO(tboldt): Disable the alarm, since there is not enough power to wake up.
            for _ in 0..5 {
                power_led_pin.set_high();
                Timer::after(Duration::from_millis(200)).await;
                power_led_pin.set_low();
                Timer::after(Duration::from_millis(100)).await;
            }
            // Exit and power down.
            break 'main;
        }

        // Run the display.
        if show_display {
            activity_led_pin.set_high();
            epaper.init().await.unwrap();
            //epaper.show_seven_color_blocks().await.unwrap();
            epaper.clear(epaper::Color::White).await.unwrap();
            epaper.deep_sleep().await.unwrap();
            activity_led_pin.set_low();
            show_display = false;
        }

        if running_on_battery {
            break 'main;
        }

        if charge_state_pin.is_low() {
            // Charging.
            power_led_pin.set_high();
        } else {
            // Not charging.
            power_led_pin.set_low();
        }

        if user_button_pin.is_low() {
            button_press_count += 1;
            if button_press_count > 3 {
                // TODO(tboldt): Restrict how requently the display can be shown.
                show_display = true;
                button_press_count = 0;
            }
        } else {
            button_press_count = 0;
        }

        watchdog.feed();
        Timer::after(Duration::from_millis(200)).await;
    }

    epd_enable_pin.set_low();

    // Disconnect the battery.
    battery_enable_pin.set_low();

    loop {
        // Should be unreachable.
        Timer::after(Duration::from_millis(1000)).await;
    }
}
