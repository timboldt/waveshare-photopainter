use rp2040_hal as hal;

use defmt::*;
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;

// NOTE: Borrowed lots of ideas and code snippets from https://github.com/tweedegolf/pcf85063a.
// Datasheet: https://www.nxp.com/docs/en/data-sheet/PCF85063A.pdf

#[derive(Debug)]
pub enum Error<E> {
    /// I2C bus error
    I2C(E),
    /// Invalid input data
    _InvalidInputData,
    /// A time component was out of range
    _ComponentRange,
}

// pub const OFFSET: u8 = 0x02;
// pub const RAM_BYTE: u8 = 0x03;
// pub const MINUTES: u8 = 0x05;
// pub const HOURS: u8 = 0x06;
// pub const DAYS: u8 = 0x07;
// pub const WEEKDAYS: u8 = 0x08;
// pub const MONTHS: u8 = 0x09;
// pub const YEARS: u8 = 0x0A;

// // alarm registers
// pub const SECOND_ALARM: u8 = 0x0B;
// pub const MINUTE_ALARM: u8 = 0x0C;
// pub const HOUR_ALARM: u8 = 0x0D;
// pub const DAY_ALARM: u8 = 0x0E;
// pub const WEEKDAY_ALARM: u8 = 0x0F;

// // timer registers
// pub const TIMER_VALUE: u8 = 0x10;
// pub const TIMER_MODE: u8 = 0x11;

const DEVICE_ADDRESS: u8 = 0b1010001;

// Control and status registers.
const REG_CONTROL_1: u8 = 0x00;
const REG_CONTROL_2: u8 = 0x01;
// Time and date registers.
const REG_SECONDS: u8 = 0x04;

// REG_CONTROL_1 values.
const CONTROL_1_DEVICE_RESET: u8 = 0x58;

// REG_SECONDS values.
const SECONDS_OSCILLATOR_STOP: u8 = 0x80;
const SECONDS_VALUE_MASK: u8 = 0x7F;

#[derive(Debug, Default)]
pub struct PCF85063<I2C> {
    /// The concrete I2C device implementation.
    i2c: I2C,
}

impl<I2C, E> PCF85063<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn new(i2c: I2C) -> Self {
        PCF85063 { i2c }
    }

    pub fn init_device(&mut self, delay: &mut hal::Timer) -> Result<(), Error<E>> {
        self.write_register(REG_CONTROL_1, CONTROL_1_DEVICE_RESET)?;
        delay.delay_ms(500);
        let sec = self.read_register(REG_SECONDS)?;
        self.write_register(REG_SECONDS, sec | SECONDS_OSCILLATOR_STOP)?;
        self.write_register(REG_CONTROL_2, 0x80)?;
        for i in 0..5 {
            let sec = self.read_register(REG_SECONDS)?;
            self.write_register(REG_SECONDS, sec & SECONDS_VALUE_MASK)?;
            if sec & 0x80 == 0 {
                break;
            }
            if i >= 4 {
                info!("RTC clock stability is unknown")
            }
            delay.delay_ms(500);
        }
        Ok(())
    }

    fn write_register(&mut self, register: u8, data: u8) -> Result<(), Error<E>> {
        let payload: [u8; 2] = [register, data];
        self.i2c.write(DEVICE_ADDRESS, &payload).map_err(Error::I2C)
    }

    fn read_register(&mut self, register: u8) -> Result<u8, Error<E>> {
        let mut data = [0];
        self.i2c
            .write_read(DEVICE_ADDRESS, &[register], &mut data)
            .map_err(Error::I2C)
            .and(Ok(data[0]))
    }
}
