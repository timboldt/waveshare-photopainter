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
    let mut led_activity_pin = Output::new(p.PIN_25, Level::Low);
    // Power LED: green.
    let _led_power_pin = Output::new(p.PIN_26, Level::High);
    // User button (low is button pressed, or the auto-switch is enabled).
    let _user_button_pin = Input::new(p.PIN_19, Pull::Up);
    // Battery power control (high is enabled; low turns off the power).
    let _battery_enable_pin = Output::new(p.PIN_18, Level::High);
    // Battery charging indicator (low is charging; high is not charging).
    let _charge_state_pin = Input::new(p.PIN_17, Pull::Up);
    // USB bus power (high means there is power).
    let vbus_state_pin = Input::new(p.PIN_24, Pull::None);
    // Mystery pin 23, aka "Power_Mode".
    let _power_mode_pin = Input::new(p.PIN_23, Pull::None);

    // If USB connected, set up logging over USB.
    if vbus_state_pin.is_high() {
        // Set up logging over USB.
        // let _ = embassy_rp::usb::init(
        //     p.USB,
        //     p.PIN_20,
        //     p.PIN_21,
        //     p.PIN_22,
        //     embassy_rp::usb::Speed::Full,
        // );
    }

    // Set up E-Paper Display
    let epd_clk = p.PIN_10;
    let epd_mosi = p.PIN_11;
    let mut epd_config = spi::Config::default();
    epd_config.frequency = 8_000_000;
    let spi = Spi::new_txonly(p.SPI1, epd_clk, epd_mosi, p.DMA_CH0, epd_config);

    let epd_reset_pin = Output::new(p.PIN_12, Level::High);
    let epd_dc_pin = Output::new(p.PIN_8, Level::High);
    let epd_cs_pin = Output::new(p.PIN_9, Level::High);
    let epd_busy_pin = Input::new(p.PIN_13, Pull::None);
    let mut epaper =
        epaper::EPaper7In3F::new(spi, epd_reset_pin, epd_dc_pin, epd_cs_pin, epd_busy_pin);

    //Enable the E-Paper Display power.
    let _ = Output::new(p.PIN_16, Level::High);

    // TODO(tboldt): Setup SD card SPI
    // #define SD_CS_PIN       5
    // #define SD_CLK_PIN      2
    // #define SD_MOSI_PIN     3
    // #define SD_MISO_PIN     4

    // TODO(tboldt): Setup Real Time Clock
    // #define RTC_SDA         14
    // #define RTC_SCL         15
    // #define RTC_INT         6

    // TODO(tboldt): Setup VBAT ADC on pin 29
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

    // epaper.init().await.unwrap();
    // epaper.show_seven_color_blocks().await.unwrap();
    // epaper.deep_sleep().await.unwrap();

    // Loop
    loop {
        // Log
        info!("LED On!");

        // Turn LED On
        led_activity_pin.set_high();

        // Wait 100ms
        Timer::after(Duration::from_millis(500)).await;

        // Log
        info!("LED Off!");

        // Turn Led Off
        led_activity_pin.set_low();

        // Wait 100ms
        Timer::after(Duration::from_millis(500)).await;

        watchdog.feed();
    }
}
