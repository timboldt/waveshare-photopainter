const PCF85063_ADDRESS: u8 = 0x51;

const CONTROL_1_REG: u8 = 0x00;
const CONTROL_2_REG: u8 = 0x01;
#[allow(dead_code)]
const OFFSET_REG: u8 = 0x02;
#[allow(dead_code)]
const RAM_BYTE_REG: u8 = 0x03;
const SECONDS_REG: u8 = 0x04;
// Time register addresses - not currently used but needed for future time reading functionality
#[allow(dead_code)]
const MINUTES_REG: u8 = 0x05;
#[allow(dead_code)]
const HOURS_REG: u8 = 0x06;
#[allow(dead_code)]
const DAYS_REG: u8 = 0x07;
#[allow(dead_code)]
const WEEKDAYS_REG: u8 = 0x08;
#[allow(dead_code)]
const MONTHS_REG: u8 = 0x09;
#[allow(dead_code)]
const YEARS_REG: u8 = 0x0A;
// Alarm and timer registers - not currently used but may be needed in the future
#[allow(dead_code)]
const SECOND_ALARM_REG: u8 = 0x0B;
#[allow(dead_code)]
const MINUTES_ALARM_REG: u8 = 0x0C;
#[allow(dead_code)]
const HOUR_ALARM_REG: u8 = 0x0D;
#[allow(dead_code)]
const DAY_ALARM_REG: u8 = 0x0E;
#[allow(dead_code)]
const WEEKDAY_ALARM_REG: u8 = 0x0F;
#[allow(dead_code)]
const TIMER_VALUE_REG: u8 = 0x10;
#[allow(dead_code)]
const TIMER_MODE_REG: u8 = 0x11;

#[allow(dead_code)]
#[derive(Debug)]
pub struct TimeData {
    pub years: u16,
    pub months: u16,
    pub days: u16,
    pub hours: u16,
    pub minutes: u16,
    pub seconds: u16,
}

// Helper functions for BCD conversion - not currently used but may be needed for time
// reading/setting functionality
#[allow(dead_code)]
fn dec_to_bcd(val: u8) -> u8 {
    ((val / 10) << 4) | (val % 10)
}

#[allow(dead_code)]
fn bcd_to_dec(val: u8) -> u8 {
    ((val >> 4) * 10) + (val & 0x0F)
}

use embassy_rp::i2c::{self, Async};
use embassy_time::Timer;

#[derive(Clone, Copy, Debug)]
pub enum Error {
    #[allow(dead_code)]
    Timeout,
    #[allow(dead_code)]
    I2cError(i2c::Error),
}

pub struct Pcf85063<I2C: embassy_rp::i2c::Instance + 'static> {
    i2c: i2c::I2c<'static, I2C, Async>,
}

impl<I2C> Pcf85063<I2C>
where
    I2C: embassy_rp::i2c::Instance,
{
    pub fn new(i2c: i2c::I2c<'static, I2C, Async>) -> Self {
        Pcf85063 { i2c }
    }

    pub async fn init(&mut self) -> Result<(), Error> {
        self.i2c
            .write_async(PCF85063_ADDRESS, [CONTROL_1_REG, 0x58])
            .await
            .map_err(Error::I2cError)?;
        Timer::after_millis(500).await;
        let mut read_sec = [0u8; 1];
        self.i2c
            .write_read_async(PCF85063_ADDRESS, [SECONDS_REG], &mut read_sec)
            .await
            .map_err(Error::I2cError)?;
        self.i2c
            .write_async(PCF85063_ADDRESS, [SECONDS_REG, read_sec[0] | 0x80])
            .await
            .map_err(Error::I2cError)?;
        self.i2c
            .write_async(PCF85063_ADDRESS, [CONTROL_2_REG, 0x80])
            .await
            .map_err(Error::I2cError)?;
        for _ in 0..5 {
            self.i2c
                .write_read_async(PCF85063_ADDRESS, [SECONDS_REG], &mut read_sec)
                .await
                .map_err(Error::I2cError)?;
            if (read_sec[0] & 0x80) == 0 {
                return Ok(());
            }
            self.i2c
                .write_async(PCF85063_ADDRESS, [SECONDS_REG, read_sec[0] & 0x7F])
                .await
                .map_err(Error::I2cError)?;
            Timer::after_millis(500).await;
        }
        Err(Error::Timeout)
    }
}
