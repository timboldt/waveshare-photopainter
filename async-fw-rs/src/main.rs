#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_rp::spi::{Config, Spi};
use embassy_time::{Duration, Timer};
use gpio::{Input, Level, Output, Pull};
use {defmt_rtt as _, panic_probe as _};

mod epaper;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialise Peripherals
    let p = embassy_rp::init(Default::default());

    let epd_clk = p.PIN_10;
    let epd_mosi = p.PIN_11;
    let mut config = Config::default();
    config.frequency = 8_000_000;
    let spi = Spi::new_txonly(
        p.SPI1,
        epd_clk,
        epd_mosi,
        p.DMA_CH0,
        config,
    );

    let _epd_enable_pin = Output::new(p.PIN_16, Level::High);
    let _battery_disable_pin = Output::new(p.PIN_18, Level::High);
    let mut led_activity = Output::new(p.PIN_25, Level::Low);
    let mut _led_power = Output::new(p.PIN_26, Level::High);

    let epd_reset_pin = Output::new(p.PIN_12, Level::Low);
    let epd_dc_pin = Output::new(p.PIN_8, Level::Low);
    let epd_cs_pin = Output::new(p.PIN_9, Level::High);
    let epd_busy_pin = Input::new(p.PIN_13, Pull::None);

    let mut epaper = epaper::EPaper7In3F::new(spi, epd_reset_pin, epd_dc_pin, epd_cs_pin, epd_busy_pin);

    Timer::after_millis(1000).await;
    epaper.init().await.unwrap();
    epaper.show_seven_color_blocks().await.unwrap();
    epaper.deep_sleep().await.unwrap();


    // Loop
    loop {
        // Log
        info!("LED On!");

        // Turn LED On
        led_activity.set_high();

        // Wait 100ms
        Timer::after(Duration::from_millis(500)).await;

        // Log
        info!("LED Off!");

        // Turn Led Off
        led_activity.set_low();

        // Wait 100ms
        Timer::after(Duration::from_millis(500)).await;
    }
}
