//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

mod rtc;

use panic_probe as _;

use rp2040_hal as hal;

use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_0_2::adc::OneShot;
use fugit::RateExtU32;
use hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

// Minimum power is 3.1V.
const MIN_BATTERY_MILLIVOLTS: u32 = 3100;

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

#[rp2040_hal::entry]
fn main() -> ! {
    info!("Boot start");

    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // watchdog_enable(8*1000, 1);    // 8s

    // See unrelease create https://github.com/Caemor/epd-waveshare.
    // spi_init(EPD_SPI_PORT, 8000 * 1000);
    // gpio_set_function(EPD_CLK_PIN, GPIO_FUNC_SPI);
    // gpio_set_function(EPD_MOSI_PIN, GPIO_FUNC_SPI);
    // DEV_GPIO_Mode(EPD_RST_PIN, 1);
    // DEV_GPIO_Mode(EPD_DC_PIN, 1);
    // DEV_GPIO_Mode(EPD_CS_PIN, 1);
    // DEV_GPIO_Mode(EPD_BUSY_PIN, 0);
    //     #define EPD_POWER_EN    16
    // DEV_GPIO_Mode(EPD_POWER_EN, 1);
    // DEV_Digital_Write(EPD_POWER_EN, 1);	// EPD power on
    // DEV_Digital_Write(EPD_CS_PIN, 1);

    // See https://github.com/rp-rs/rp-hal-boards/blob/main/boards/rp-pico/examples/pico_spi_sd_card.rs.
    // spi_init(SD_SPI_PORT, 12500 * 1000);
    // gpio_set_function(SD_CLK_PIN, GPIO_FUNC_SPI);
    // gpio_set_function(SD_MOSI_PIN, GPIO_FUNC_SPI);
    // gpio_set_function(SD_MISO_PIN, GPIO_FUNC_SPI);
    // DEV_GPIO_Mode(SD_CS_PIN, 1);

    let sda_pin: hal::gpio::Pin<_, hal::gpio::FunctionI2C, _> = pins.gpio14.reconfigure();
    let scl_pin: hal::gpio::Pin<_, hal::gpio::FunctionI2C, _> = pins.gpio15.reconfigure();

    let i2c = hal::I2C::i2c1(
        pac.I2C1,
        sda_pin,
        scl_pin,
        400.kHz(),
        &mut pac.RESETS,
        &clocks.peripheral_clock,
    );

    let mut rtc = rtc::PCF85063::new(i2c);
    rtc.init_device(&mut delay).unwrap();

    // RTC alarm (low means it triggered)
    let mut rtc_alarm = pins.gpio6.into_pull_up_input();
    info!("Alarm triggered: {}", rtc_alarm.is_low().unwrap());

    // Set up ADC, which is used to read the battery voltage.
    let mut adc = hal::Adc::new(pac.ADC, &mut pac.RESETS);
    let mut vbat_adc = hal::adc::AdcPin::new(pins.gpio29).unwrap();

    // Activity LED (red).
    let mut activity_led = pins.gpio25.into_push_pull_output();

    // Power LED (green).
    let mut power_led = pins.gpio26.into_push_pull_output();

    // Battery power control (high is enabled; low turns off the power).
    let mut battery_enable = pins.gpio18.into_push_pull_output();

    // User button (low is button pressed, or the auto-switch is enabled).
    let mut user_button = pins.gpio19.into_pull_up_input();

    // Battery charging indicator (low is charging; high is not charging).
    let mut charge_state = pins.gpio17.into_pull_up_input();

    // USB bus power (high means there is power).
    let mut vbus_state = pins.gpio24.into_floating_input();

    activity_led.set_low().unwrap();
    power_led.set_low().unwrap();

    // Connect the battery.
    battery_enable.set_high().unwrap();

    delay.delay_ms(500);
    let battery: u16 = adc.read(&mut vbat_adc).unwrap();
    // Some sort of voltage divider (10x?) at 3.3V reference, x1000 for mV, using a 12-bit ADC.
    // XXXX for some reason, Waveshare uses a 3x multiplier in their code and it seems to work. Why?
    let battery_millivolts = battery as u32 * 10 * 3300 / (1 << 12);

    info!("VBUS power: {}", vbus_state.is_high().unwrap());
    info!("Charging: {}", charge_state.is_low().unwrap());
    info!("voltage: {} mV", battery_millivolts);

    // let mut temperature_sensor = adc.take_temp_sensor().unwrap();
    // for i in 0..10 {
    //     let temp_sens_adc_counts: i64 = adc.read(&mut temperature_sensor).unwrap();
    //     info!("Temperature: {} cnts", temp_sens_adc_counts);
    //     let temp_uv = temp_sens_adc_counts * 3300 * 1000 / (1 << 12);
    //     info!("Temperature: {} uV", temp_uv);
    //     let temperature = 27 - (temp_uv - 706 * 1000) * 581 / 1000 / 1000;
    //     info!("Temperature: {}", temperature);
    //     delay.delay_ms(100);
    // }

    // rtcRunAlarm(Time, alarmTime);  // RTC run alarm

    //  sdScanDir();

    // void run_display(Time_data Time, Time_data alarmTime, char hasCard)
    // {
    //     if(hasCard) {
    //         setFilePath();
    //         EPD_7in3f_display_BMP(pathName, measureVBAT());   // display bmp
    //     }
    //     else {
    //         EPD_7in3f_display(measureVBAT());
    //     }

    //     PCF85063_clear_alarm_flag();    // clear RTC alarm flag
    //     rtcRunAlarm(Time, alarmTime);  // RTC run alarm
    // }

    info!("Init done");

    if vbus_state.is_low().unwrap() {
        info!("Running on batteries");

        if (battery_millivolts > MIN_BATTERY_MILLIVOLTS) {
            // XXX run display; in the meantime, show the red light so we know we are here.
            activity_led.set_high().unwrap();
            delay.delay_ms(500);
        } else {
            info!("Low power");
            // XXX disable alarm
            for _ in 0..5 {
                power_led.set_high().unwrap();
                delay.delay_ms(200);
                power_led.set_low().unwrap();
                delay.delay_ms(100);
            }
        }
    } else {
        info!("Running off VBUS power");

        // As long as it is plugged in, just keep looping.
        while vbus_state.is_high().unwrap() {
            if charge_state.is_low().unwrap() {
                // Charging.
                power_led.set_high().unwrap();
            } else {
                // Not charging.
                power_led.set_low().unwrap();
            }

            if user_button.is_low().unwrap() {
                // TODO: also handle RTC when on USB power: `|| rtc_alarm.is_low().unwrap() {`.
                // xxx run display; in the meantime, show the red light so we know we are here.
                activity_led.set_high().unwrap();
                info!("Button pushed");
                delay.delay_ms(500);
                activity_led.set_low().unwrap();
            }

            delay.delay_ms(200);
        }
    }

    // Disconnect the battery.
    battery_enable.set_low().unwrap();

    loop {
        // Should be unreachable.
        delay.delay_ms(1000);
    }
}
