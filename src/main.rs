#![no_std]
#![no_main]

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

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::{
    adc::{self, Adc, Channel, InterruptHandler},
    bind_interrupts,
    clocks::RoscRng,
    gpio,
    spi::{self, Spi},
    watchdog::*,
};
use embassy_time::{Duration, Timer};
use gpio::{Input, Level, Output, Pull};
use graphics::draw_random_walk_art;
use panic_probe as _;
use rand::RngCore;

mod epaper;
mod graphics;

// Minimum power is 3.1V.
const MIN_BATTERY_MILLIVOLTS: u32 = 3100;

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

// struct DummyTimesource();

// // TODO(tboldt): Implement the TimeSource trait with the RTC.
// impl embedded_sdmmc::TimeSource for DummyTimesource {
//     fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
//         embedded_sdmmc::Timestamp {
//             year_since_1970: 0,
//             zero_indexed_month: 0,
//             zero_indexed_day: 0,
//             hours: 0,
//             minutes: 0,
//             seconds: 0,
//         }
//     }
// }

//static mut DISPLAY_BUF: epaper::DisplayBuffer = epaper::DisplayBuffer::default();

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

    // // Setup SD card SPI
    // let sdcard_clk = p.PIN_2;
    // let sdcard_mosi = p.PIN_3;
    // let sdcard_miso = p.PIN_4;
    // let mut sdcard_config = spi::Config::default();

    // // Before talking to the SD Card, the caller needs to send 74 clocks cycles on the SPI Clock
    // line, at 400 kHz, with no chip-select asserted (or at least, not the chip-select of the SD
    // Card). sdcard_config.frequency = 400_000;
    // let sdcard_spi = Spi::new_blocking(p.SPI0, sdcard_clk, sdcard_mosi, sdcard_miso,
    // sdcard_config);

    // // Use a dummy cs pin here, for embedded-hal SpiDevice compatibility reasons
    // let sdcard_spi_dev = ExclusiveDevice::new_no_delay(sdcard_spi, DummyCsPin);
    // // Real cs pin
    // let sdcard_cs_pin = Output::new(p.PIN_5, Level::High);

    // let sdcard = SdCard::new(sdcard_spi_dev, sdcard_cs_pin, embassy_time::Delay);

    // //Once the card is initialized, the SPI clock can go faster.
    // let mut sdcard_config = spi::Config::default();
    // sdcard_config.frequency = 12_500_000;
    // sdcard
    //     .spi(|dev| dev.bus_mut().set_config(&sdcard_config))
    //     .ok();
    // let mut volume_mgr = embedded_sdmmc::VolumeManager::new(sdcard, DummyTimesource());
    // let mut volume0 = volume_mgr
    //     .open_volume(embedded_sdmmc::VolumeIdx(0))
    //     .unwrap();
    // let mut root_dir = volume0.open_root_dir().unwrap();
    // let pic_dir = root_dir.open_dir("pic").unwrap();
    // //xxx iterate_dir()

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

    info!("Init done");

    let mut show_display = true;
    let mut button_press_count = 0;
    'main: loop {
        let running_on_battery = vbus_state_pin.is_low();
        info!("Running on battery? {}", running_on_battery);

        // If the battery is low, flash the low power LED, disable the alarm, and turn off the
        // power.
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
            epaper.init(&mut watchdog).await.unwrap();
            let display_buf = epaper::DisplayBuffer::get();
            draw_random_walk_art(display_buf, rng.next_u64()).unwrap();
            epaper.show_image(display_buf, &mut watchdog).await.unwrap();
            //epaper.show_seven_color_blocks(&mut watchdog).await.unwrap();
            //epaper.clear(epaper::Color::White, &mut watchdog).await.unwrap();
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
