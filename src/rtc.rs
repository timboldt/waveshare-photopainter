const PCF85063_ADDRESS: u8 = 0x51;

const CONTROL_1_REG: u8 = 0x00;
const CONTROL_2_REG: u8 = 0x01;
#[allow(dead_code)]
const OFFSET_REG: u8 = 0x02;
#[allow(dead_code)]
const RAM_BYTE_REG: u8 = 0x03;
const SECONDS_REG: u8 = 0x04;
const MINUTES_REG: u8 = 0x05;
const HOURS_REG: u8 = 0x06;
const DAYS_REG: u8 = 0x07;
#[allow(dead_code)]
const WEEKDAYS_REG: u8 = 0x08;
const MONTHS_REG: u8 = 0x09;
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
#[derive(Debug, Clone, Copy)]
pub struct TimeData {
    pub years: u16,
    pub months: u16,
    pub days: u16,
    pub hours: u16,
    pub minutes: u16,
    pub seconds: u16,
}

// Helper functions for BCD conversion
fn dec_to_bcd(val: u8) -> u8 {
    ((val / 10) << 4) | (val % 10)
}

fn bcd_to_dec(val: u8) -> u8 {
    ((val >> 4) * 10) + (val & 0x0F)
}

/// Add seconds to a time, handling overflow
pub fn add_seconds_to_time(time: &TimeData, seconds_to_add: u32) -> TimeData {
    let mut result = *time;

    result.seconds += seconds_to_add as u16;

    // Handle seconds overflow
    if result.seconds >= 60 {
        result.minutes += result.seconds / 60;
        result.seconds %= 60;
    }

    // Handle minutes overflow
    if result.minutes >= 60 {
        result.hours += result.minutes / 60;
        result.minutes %= 60;
    }

    // Handle hours overflow
    if result.hours >= 24 {
        result.days += result.hours / 24;
        result.hours %= 24;
    }

    // Handle days overflow (simplified - doesn't account for month lengths)
    // For short sleep durations this should be fine
    if result.days > 31 {
        result.months += result.days / 31;
        result.days = ((result.days - 1) % 31) + 1;
    }

    if result.months > 12 {
        result.years += result.months / 12;
        result.months = ((result.months - 1) % 12) + 1;
    }

    result
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
        // Write to Control_1 register
        // NOTE: Original C code used 0x58 which includes SR bit (software reset)
        // We use 0x48 to avoid resetting the time on every init
        // 0x48 = 0b01001000 (reserved bit 6 set, CIE bit 3 set, no reset)
        self.i2c
            .write_async(PCF85063_ADDRESS, [CONTROL_1_REG, 0x48])
            .await
            .map_err(Error::I2cError)?;
        Timer::after_millis(500).await;

        // Clear alarm flag and disable alarm interrupt in Control_2
        // This prevents boot loops after waking from sleep
        // Bit 7 (AIE): Alarm Interrupt Enable - set to 0 (disabled)
        // Bit 6 (AF): Alarm Flag - cleared by writing 0
        let mut ctrl2 = [0u8; 1];
        self.i2c
            .write_read_async(PCF85063_ADDRESS, [CONTROL_2_REG], &mut ctrl2)
            .await
            .map_err(Error::I2cError)?;
        self.i2c
            .write_async(PCF85063_ADDRESS, [CONTROL_2_REG, ctrl2[0] & 0x3F])
            .await
            .map_err(Error::I2cError)?;

        // Check and clear the OS (Oscillator Stop) bit in seconds register
        let mut read_sec = [0u8; 1];
        self.i2c
            .write_read_async(PCF85063_ADDRESS, [SECONDS_REG], &mut read_sec)
            .await
            .map_err(Error::I2cError)?;
        self.i2c
            .write_async(PCF85063_ADDRESS, [SECONDS_REG, read_sec[0] | 0x80])
            .await
            .map_err(Error::I2cError)?;

        // Wait for oscillator to stabilize
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

    /// Set countdown timer for N seconds (1-255)
    /// Timer runs at 1Hz when seconds > 0
    #[allow(dead_code)]
    pub async fn set_timer(&mut self, seconds: u8) -> Result<(), Error> {
        if seconds == 0 {
            return Err(Error::Timeout); // Use Timeout as generic error
        }

        // Disable timer first
        self.i2c
            .write_async(PCF85063_ADDRESS, [TIMER_MODE_REG, 0x00])
            .await
            .map_err(Error::I2cError)?;

        // Set timer value
        self.i2c
            .write_async(PCF85063_ADDRESS, [TIMER_VALUE_REG, seconds])
            .await
            .map_err(Error::I2cError)?;

        // Enable timer: TIE=1 (interrupt enable), TI_TP=0 (timer mode), TCF=1Hz
        // Timer mode: 0b10000010 = 0x82
        // Bit 7: TIE (Timer Interrupt Enable)
        // Bit 4: TI_TP (0=timer, 1=pulse)
        // Bit 1-0: TCF (timer clock frequency: 00=4096Hz, 01=64Hz, 10=1Hz, 11=1/60Hz)
        self.i2c
            .write_async(PCF85063_ADDRESS, [TIMER_MODE_REG, 0x82])
            .await
            .map_err(Error::I2cError)?;

        // Clear timer flag in Control_2
        let mut ctrl2 = [0u8; 1];
        self.i2c
            .write_read_async(PCF85063_ADDRESS, [CONTROL_2_REG], &mut ctrl2)
            .await
            .map_err(Error::I2cError)?;

        // Clear TF (timer flag, bit 3) by writing 0 to it
        self.i2c
            .write_async(PCF85063_ADDRESS, [CONTROL_2_REG, ctrl2[0] & !0x08])
            .await
            .map_err(Error::I2cError)?;

        Ok(())
    }

    /// Clear timer interrupt flag
    #[allow(dead_code)]
    pub async fn clear_timer_flag(&mut self) -> Result<(), Error> {
        let mut ctrl2 = [0u8; 1];
        self.i2c
            .write_read_async(PCF85063_ADDRESS, [CONTROL_2_REG], &mut ctrl2)
            .await
            .map_err(Error::I2cError)?;

        // Clear TF (timer flag, bit 3)
        self.i2c
            .write_async(PCF85063_ADDRESS, [CONTROL_2_REG, ctrl2[0] & !0x08])
            .await
            .map_err(Error::I2cError)?;

        Ok(())
    }

    /// Disable timer
    #[allow(dead_code)]
    pub async fn disable_timer(&mut self) -> Result<(), Error> {
        self.i2c
            .write_async(PCF85063_ADDRESS, [TIMER_MODE_REG, 0x00])
            .await
            .map_err(Error::I2cError)?;
        Ok(())
    }

    /// Read current time from RTC
    pub async fn get_time(&mut self) -> Result<TimeData, Error> {
        let mut buf = [0u8; 7];
        self.i2c
            .write_read_async(PCF85063_ADDRESS, [SECONDS_REG], &mut buf)
            .await
            .map_err(Error::I2cError)?;

        Ok(TimeData {
            seconds: bcd_to_dec(buf[0] & 0x7F) as u16,
            minutes: bcd_to_dec(buf[1] & 0x7F) as u16,
            hours: bcd_to_dec(buf[2] & 0x3F) as u16,
            days: bcd_to_dec(buf[3] & 0x3F) as u16,
            months: bcd_to_dec(buf[5] & 0x1F) as u16,
            years: bcd_to_dec(buf[6]) as u16 + 2000,
        })
    }

    /// Set time on RTC
    /// Matches the original Waveshare C implementation which writes each register individually
    pub async fn set_time(&mut self, time: &TimeData) -> Result<(), Error> {
        let years = if time.years >= 2000 {
            time.years - 2000
        } else {
            time.years
        };

        // Validate year is in valid range for RTC (0-99)
        if years > 99 {
            return Err(Error::Timeout); // Use Timeout as generic error
        }

        // Write each register individually, matching the C implementation
        // This is critical - the RTC doesn't support multi-byte writes to time registers

        // Set HMS (Hours, Minutes, Seconds)
        self.i2c
            .write_async(
                PCF85063_ADDRESS,
                [HOURS_REG, dec_to_bcd(time.hours as u8) & 0x3F],
            )
            .await
            .map_err(Error::I2cError)?;

        self.i2c
            .write_async(
                PCF85063_ADDRESS,
                [MINUTES_REG, dec_to_bcd(time.minutes as u8) & 0x7F],
            )
            .await
            .map_err(Error::I2cError)?;

        self.i2c
            .write_async(
                PCF85063_ADDRESS,
                [SECONDS_REG, dec_to_bcd(time.seconds as u8) & 0x7F],
            )
            .await
            .map_err(Error::I2cError)?;

        // Set YMD (Years, Months, Days)
        self.i2c
            .write_async(PCF85063_ADDRESS, [YEARS_REG, dec_to_bcd(years as u8)])
            .await
            .map_err(Error::I2cError)?;

        self.i2c
            .write_async(
                PCF85063_ADDRESS,
                [MONTHS_REG, dec_to_bcd(time.months as u8) & 0x1F],
            )
            .await
            .map_err(Error::I2cError)?;

        self.i2c
            .write_async(
                PCF85063_ADDRESS,
                [DAYS_REG, dec_to_bcd(time.days as u8) & 0x3F],
            )
            .await
            .map_err(Error::I2cError)?;

        Ok(())
    }

    /// Enable alarm at specific time
    /// Matches the original Waveshare C implementation which writes each register individually
    pub async fn set_alarm(&mut self, alarm_time: &TimeData) -> Result<(), Error> {
        // Enable alarm interrupt in Control_2
        let mut ctrl2 = [0u8; 1];
        self.i2c
            .write_read_async(PCF85063_ADDRESS, [CONTROL_2_REG], &mut ctrl2)
            .await
            .map_err(Error::I2cError)?;

        self.i2c
            .write_async(PCF85063_ADDRESS, [CONTROL_2_REG, ctrl2[0] | 0x80])
            .await
            .map_err(Error::I2cError)?;

        // Set alarm time - write each register individually, matching the C implementation
        // Note: C code writes in order: DAY, HOUR, MINUTES, SECOND
        self.i2c
            .write_async(
                PCF85063_ADDRESS,
                [DAY_ALARM_REG, dec_to_bcd(alarm_time.days as u8) & 0x7F],
            )
            .await
            .map_err(Error::I2cError)?;

        self.i2c
            .write_async(
                PCF85063_ADDRESS,
                [HOUR_ALARM_REG, dec_to_bcd(alarm_time.hours as u8) & 0x7F],
            )
            .await
            .map_err(Error::I2cError)?;

        self.i2c
            .write_async(
                PCF85063_ADDRESS,
                [
                    MINUTES_ALARM_REG,
                    dec_to_bcd(alarm_time.minutes as u8) & 0x7F,
                ],
            )
            .await
            .map_err(Error::I2cError)?;

        self.i2c
            .write_async(
                PCF85063_ADDRESS,
                [
                    SECOND_ALARM_REG,
                    dec_to_bcd(alarm_time.seconds as u8) & 0x7F,
                ],
            )
            .await
            .map_err(Error::I2cError)?;

        Ok(())
    }

    /// Clear alarm flag
    pub async fn clear_alarm_flag(&mut self) -> Result<(), Error> {
        let mut ctrl2 = [0u8; 1];
        self.i2c
            .write_read_async(PCF85063_ADDRESS, [CONTROL_2_REG], &mut ctrl2)
            .await
            .map_err(Error::I2cError)?;

        // Clear AF (alarm flag, bit 6)
        self.i2c
            .write_async(PCF85063_ADDRESS, [CONTROL_2_REG, ctrl2[0] & !0x40])
            .await
            .map_err(Error::I2cError)?;

        Ok(())
    }

    /// Disable alarm
    #[allow(dead_code)]
    pub async fn disable_alarm(&mut self) -> Result<(), Error> {
        // Disable each alarm register by setting bit 7
        let mut alarm_regs = [0u8; 4];
        self.i2c
            .write_read_async(PCF85063_ADDRESS, [SECOND_ALARM_REG], &mut alarm_regs)
            .await
            .map_err(Error::I2cError)?;

        self.i2c
            .write_async(
                PCF85063_ADDRESS,
                [
                    SECOND_ALARM_REG,
                    alarm_regs[0] | 0x80,
                    alarm_regs[1] | 0x80,
                    alarm_regs[2] | 0x80,
                    alarm_regs[3] | 0x80,
                ],
            )
            .await
            .map_err(Error::I2cError)?;

        // Disable alarm interrupt in Control_2
        let mut ctrl2 = [0u8; 1];
        self.i2c
            .write_read_async(PCF85063_ADDRESS, [CONTROL_2_REG], &mut ctrl2)
            .await
            .map_err(Error::I2cError)?;

        self.i2c
            .write_async(PCF85063_ADDRESS, [CONTROL_2_REG, ctrl2[0] & 0x7F])
            .await
            .map_err(Error::I2cError)?;

        Ok(())
    }
}
