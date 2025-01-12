const PCF85063_ADDRESS: u8 = 0x51;

const CONTROL_1_REG: u8 = 0x00;
const CONTROL_2_REG: u8 = 0x01;
const OFFSET_REG: u8 = 0x02;
const RAM_BYTE_REG: u8 = 0x03;
const SECONDS_REG: u8 = 0x04;
const MINUTES_REG: u8 = 0x05;
const HOURS_REG: u8 = 0x06;
const DAYS_REG: u8 = 0x07;
const WEEKDAYS_REG: u8 = 0x08;
const MONTHS_REG: u8 = 0x09;
const YEARS_REG: u8 = 0x0A;
const SECOND_ALARM_REG: u8 = 0x0B;
const MINUTES_ALARM_REG: u8 = 0x0C;
const HOUR_ALARM_REG: u8 = 0x0D;
const DAY_ALARM_REG: u8 = 0x0E;
const WEEKDAY_ALARM_REG: u8 = 0x0F;
const TIMER_VALUE_REG: u8 = 0x10;
const TIMER_MODE_REG: u8 = 0x11;

#[derive(Debug)]
pub struct TimeData {
    pub years: u16,
    pub months: u16,
    pub days: u16,
    pub hours: u16,
    pub minutes: u16,
    pub seconds: u16,
}

fn dec_to_bcd(val: u8) -> u8 {
    ((val / 10) << 4) | (val % 10)
}

fn bcd_to_dec(val: u8) -> u8 {
    ((val >> 4) * 10) + (val & 0x0F)
}

// void PCF85063_SetTime_YMD(int Years,int Months,int Days);
// void PCF85063_SetTime_HMS(int hour,int minute,int second);
// Time_data PCF85063_GetTime();
// void PCF85063_alarm_Time_Enabled(Time_data time);
// void PCF85063_alarm_Time_Disable();
// int PCF85063_get_alarm_flag();
// void PCF85063_clear_alarm_flag();
// void PCF85063_test();
// void rtcRunAlarm(Time_data time, Time_data alarmTime);

// #endif

// /*****************************************************************************
// * | File      	:   waveshare_PCF85063.c
// * | Author      :   Waveshare team
// * | Function    :   PCF85063 driver
// * | Info        :
// *----------------
// * |	This version:   V1.0
// * | Date        :   2021-02-02
// * | Info        :   Basic version
// *
// ******************************************************************************/
// #include "DEV_Config.h"
// #include "waveshare_PCF85063.h"

// /******************************************************************************
// function:	Read one byte of data to EMC2301 via I2C
// parameter:
//             Addr: Register address
// Info:
// ******************************************************************************/
// static UBYTE PCF85063_Read_Byte(UBYTE Addr)
// {
// 	return I2C_Read_Byte(Addr);
// }

// /******************************************************************************
// function:	Send one byte of data to EMC2301 via I2C
// parameter:
//             Addr: Register address
//            Value: Write to the value of the register
// Info:
// ******************************************************************************/
// static void PCF85063_Write_Byte(UBYTE Addr, UBYTE Value)
// {
// 	I2C_Write_Byte(Addr, Value);
// }

// int DecToBcd(int val)
// {
// 	return ((val/10)*16 + (val%10));
// }

// int BcdToDec(int val)
// {
// 	return ((val/16)*10 + (val%16));
// }

// void PCF85063_init()
// {
// 	int inspect = 0;
// 	PCF85063_Write_Byte(CONTROL_1_REG,0x58);
// 	DEV_Delay_ms(500);
// 	PCF85063_Write_Byte(SECONDS_REG,PCF85063_Read_Byte(SECONDS_REG)|0x80);
// 	PCF85063_Write_Byte(CONTROL_2_REG,0x80);
// 	while(1)
// 	{
// 		PCF85063_Write_Byte(SECONDS_REG,PCF85063_Read_Byte(SECONDS_REG)&0x7F);
// 		if((PCF85063_Read_Byte(SECONDS_REG)&0x80) == 0)
// 		break;
// 		DEV_Delay_ms(500);
// 		inspect  = inspect+1;
// 		if(inspect>5)
// 		{
// 			printf("Clock stability unknown\r\n");
// 			break;
// 		}
// 	}
// }

// void PCF85063_SetTime_YMD(int Years,int Months,int Days)
// {
// 	if(Years>99)
// 		Years = 99;
// 	if(Months>12)
// 		Months = 12;
// 	if(Days>31)
// 		Days = 31;
// 	PCF85063_Write_Byte(YEARS_REG  ,DecToBcd(Years));
// 	PCF85063_Write_Byte(MONTHS_REG ,DecToBcd(Months)&0x1F);
// 	PCF85063_Write_Byte(DAYS_REG   ,DecToBcd(Days)&0x3F);
// }

// void PCF85063_SetTime_HMS(int hour,int minute,int second)
// {
// 	if(hour>23)
// 		hour = 23;
// 	if(minute>59)
// 		minute = 59;
// 	if(second>59)
// 		second = 59;
// 	PCF85063_Write_Byte(HOURS_REG  ,DecToBcd(hour)&0x3F);
// 	PCF85063_Write_Byte(MINUTES_REG,DecToBcd(minute)&0x7F);
// 	PCF85063_Write_Byte(SECONDS_REG,DecToBcd(second)&0x7F);
// }

// Time_data PCF85063_GetTime()
// {
// 	Time_data time;
// 	time.years = BcdToDec(PCF85063_Read_Byte(YEARS_REG));
// 	time.months = BcdToDec(PCF85063_Read_Byte(MONTHS_REG)&0x1F);
// 	time.days = BcdToDec(PCF85063_Read_Byte(DAYS_REG)&0x3F);
// 	time.hours = BcdToDec(PCF85063_Read_Byte(HOURS_REG)&0x3F);
// 	time.minutes = BcdToDec(PCF85063_Read_Byte(MINUTES_REG)&0x7F);
// 	time.seconds = BcdToDec(PCF85063_Read_Byte(SECONDS_REG)&0x7F);
// 	return time;
// }

// void PCF85063_alarm_Time_Enabled(Time_data time)
// {
//     if(time.seconds>59)
//     {
//         time.seconds = time.seconds - 60;
//         time.minutes = time.minutes + 1;
//     }
//     if(time.minutes>59)
//     {
//         time.minutes = time.minutes - 60;
//         time.hours = time.hours + 1;
//     }
//     if(time.hours>23)
//     {
//         time.hours = time.hours - 24;
//         time.days = time.days + 1;
//     }
//     if(time.months == 1 || time.months == 3 || time.months == 5 || time.months == 7 ||
// time.months == 8 || time.months == 10 || time.months == 12)     {
//         if(time.days>31)
//         {
//             time.days = time.days - 31;
//         }
//     }
//     else if(time.months == 2)
//     {
//         if(time.years%4==0)
//         {
//             if(time.days>29)
//             {
//                 time.days = time.days - 29;
//             }
//         }
//         else
//         {
//             if(time.days>28)
//             {
//                 time.days = time.days - 28;
//             }
//         }
//     }
//     else
//     {
//         if(time.days>30)
//         {
//             time.days = time.days - 30;
//         }
//     }
//     // printf("%d-%d-%d
// %d:%d:%d\r\n",time.years,time.months,time.days,time.hours,time.minutes,time.seconds);
// 	PCF85063_Write_Byte(CONTROL_2_REG, PCF85063_Read_Byte(CONTROL_2_REG)|0x80);	// Alarm on
// 	PCF85063_Write_Byte(DAY_ALARM_REG, DecToBcd(time.days) & 0x7F);
//     PCF85063_Write_Byte(HOUR_ARARM_REG, DecToBcd(time.hours) & 0x7F);
// 	PCF85063_Write_Byte(MINUTES_ALARM_REG, DecToBcd(time.minutes) & 0x7F);
// 	PCF85063_Write_Byte(SECOND_ALARM_REG, DecToBcd(time.seconds) & 0x7F);
// }

// void PCF85063_alarm_Time_Disable()
// {
// 	PCF85063_Write_Byte(HOUR_ARARM_REG   ,PCF85063_Read_Byte(HOUR_ARARM_REG)|0x80);
// 	PCF85063_Write_Byte(MINUTES_ALARM_REG,PCF85063_Read_Byte(MINUTES_ALARM_REG)|0x80);
// 	PCF85063_Write_Byte(SECOND_ALARM_REG ,PCF85063_Read_Byte(SECOND_ALARM_REG)|0x80);
// 	PCF85063_Write_Byte(DAY_ALARM_REG, PCF85063_Read_Byte(DAY_ALARM_REG)|0x80);
//     PCF85063_Write_Byte(CONTROL_2_REG   ,PCF85063_Read_Byte(CONTROL_2_REG)&0x7F);	// Alarm OFF
// }

// int PCF85063_get_alarm_flag()
// {
// 	if(PCF85063_Read_Byte(CONTROL_2_REG)&0x40 == 0x40)
// 		return 1;
// 	else
// 		return 0;
// }

// void PCF85063_clear_alarm_flag()
// {
// 	PCF85063_Write_Byte(CONTROL_2_REG   ,PCF85063_Read_Byte(CONTROL_2_REG)&0xBF);
// }

// void PCF85063_test()
// {
//     int count = 0;

// 	// PCF85063_SetTime_YMD(21,2,28);
// 	// PCF85063_SetTime_HMS(23,59,58);
// 	while(1)
// 	{
// 		Time_data T;
// 		T = PCF85063_GetTime();
// 		printf("%d-%d-%d %d:%d:%d\r\n",T.years,T.months,T.days,T.hours,T.minutes,T.seconds);
// 		count+=1;
// 		DEV_Delay_ms(1000);
// 		if(count>20)
// 		break;
// 	}
// }

// void rtcRunAlarm(Time_data time, Time_data alarmTime)
// {
//     PCF85063_SetTime_HMS(time.hours, time.minutes, time.seconds);
// 	PCF85063_SetTime_YMD(time.years, time.months, time.days);

//     PCF85063_alarm_Time_Enabled(alarmTime);
// }

use embassy_rp::i2c::{self, Async};
use embassy_time::Timer;

#[derive(Clone, Copy, Debug)]
pub enum Error {
    #[allow(dead_code)]
    Timeout,
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
