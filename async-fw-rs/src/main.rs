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

    let miso = p.PIN_12;
    let mosi = p.PIN_11;
    let clk = p.PIN_10;

    let mut spi = Spi::new(
        p.SPI1,
        clk,
        mosi,
        miso,
        p.DMA_CH0,
        p.DMA_CH1,
        Config::default(),
    );
    let mut epaper = epaper::EPaper7In3F::new(
        Output::new(p.PIN_13, Level::Low),
        Output::new(p.PIN_14, Level::Low),
        Output::new(p.PIN_15, Level::Low),
        Input::new(p.PIN_16, Pull::Up), //XXXX
        spi,
    );

    epaper.init().await.unwrap();
    epaper.show_seven_color_blocks().await.unwrap();
    epaper.deep_sleep().await.unwrap();

    // Create LED
    let mut led = Output::new(p.PIN_25, Level::Low);

    // Loop
    loop {
        // Log
        info!("LED On!");

        // Turn LED On
        led.set_high();

        // Wait 100ms
        Timer::after(Duration::from_millis(500)).await;

        // Log
        info!("LED Off!");

        // Turn Led Off
        led.set_low();

        // Wait 100ms
        Timer::after(Duration::from_millis(500)).await;
    }
}
