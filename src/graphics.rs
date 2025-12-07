use embedded_graphics::{
    mono_font::{
        ascii::{FONT_10X20, FONT_6X10, FONT_9X18},
        MonoTextStyle,
    },
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
    text::{Alignment, Text},
};
use rand::{rngs::SmallRng, Rng, SeedableRng};

use crate::{
    epaper::{DisplayBuffer, EPD_7IN3F_HEIGHT, EPD_7IN3F_WIDTH},
    rtc::TimeData,
};

pub fn draw_random_walk_art(
    display: &mut DisplayBuffer,
    seed: u64,
) -> Result<(), core::convert::Infallible> {
    let mut rng = SmallRng::seed_from_u64(seed);

    let background = if rng.gen_range(0..6) == 0 {
        Rgb888::BLACK
    } else {
        Rgb888::WHITE
    };
    let colors = if background == Rgb888::BLACK {
        [Rgb888::WHITE, Rgb888::YELLOW, Rgb888::CSS_ORANGE]
    } else {
        [Rgb888::RED, Rgb888::BLUE, Rgb888::GREEN]
    };
    display.clear(background).unwrap();

    let mut start_x = EPD_7IN3F_WIDTH as i32 / 2;
    let mut start_y = EPD_7IN3F_HEIGHT as i32 / 2;
    for color in colors {
        let line_style = PrimitiveStyle::with_stroke(color, 3);
        let mut p = Point::new(start_x, start_y);
        for _ in 0..2000 {
            let prev_p = p;
            let r = rng.gen_range(0..4);
            let step_size = 6;
            match r {
                0 => p.x += step_size,
                1 => p.x -= step_size,
                2 => p.y += step_size,
                3 => p.y -= step_size,
                _ => (),
            }
            Line::new(prev_p, p)
                .into_styled(line_style)
                .draw(display)
                .unwrap();
            // Stop drawing if we go out of bounds.
            if p.x < 0 || p.x >= EPD_7IN3F_WIDTH as i32 || p.y < 0 || p.y >= EPD_7IN3F_HEIGHT as i32
            {
                break;
            }
        }
        // Shift each color over a bit to get a nice overlap effect.
        start_x += 2;
        start_y += 2;
    }

    Ok(())
}

/// Quotes for display (will eventually come from SD card)
const QUOTES: [&str; 3] = [
    "The only way to do great work is to love what you do. - Steve Jobs",
    "In the middle of difficulty lies opportunity. - Albert Einstein",
    "Life is what happens when you're busy making other plans. - John Lennon",
];

/// Select a quote based on the day of year
pub fn select_quote(day_of_year: u16) -> &'static str {
    QUOTES[(day_of_year as usize) % QUOTES.len()]
}

/// Get day of week name (0 = Sunday)
fn day_of_week_name(day: u8) -> &'static str {
    match day {
        0 => "SUNDAY",
        1 => "MONDAY",
        2 => "TUESDAY",
        3 => "WEDNESDAY",
        4 => "THURSDAY",
        5 => "FRIDAY",
        6 => "SATURDAY",
        _ => "UNKNOWN",
    }
}

/// Get month name
fn month_name(month: u16) -> &'static str {
    match month {
        1 => "JANUARY",
        2 => "FEBRUARY",
        3 => "MARCH",
        4 => "APRIL",
        5 => "MAY",
        6 => "JUNE",
        7 => "JULY",
        8 => "AUGUST",
        9 => "SEPTEMBER",
        10 => "OCTOBER",
        11 => "NOVEMBER",
        12 => "DECEMBER",
        _ => "UNKNOWN",
    }
}

/// Calculate day of week using Zeller's congruence (0 = Sunday)
fn calculate_day_of_week(year: u16, month: u16, day: u16) -> u8 {
    let mut y = year;
    let mut m = month;

    // Adjust for Zeller's (March = 3, Feb = 14 of previous year)
    if m < 3 {
        m += 12;
        y -= 1;
    }

    let q = day;
    let k = y % 100;
    let j = y / 100;

    let h = (q + ((13 * (m + 1)) / 5) + k + (k / 4) + (j / 4) - (2 * j)) % 7;

    // Convert Zeller's result (0=Saturday) to our format (0=Sunday)
    ((h + 6) % 7) as u8
}

/// Calculate day of year (1-366)
fn day_of_year(month: u16, day: u16, year: u16) -> u16 {
    let days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let is_leap = (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400);

    let mut days = day;
    for m in 1..month {
        days += days_in_month[(m - 1) as usize];
        if m == 2 && is_leap {
            days += 1;
        }
    }
    days
}

/// Draw a calendar page with date and quote
pub fn draw_calendar_page(
    display: &mut DisplayBuffer,
    time: &TimeData,
    seed: u64,
) -> Result<(), crate::epaper::Error> {
    let _rng = SmallRng::seed_from_u64(seed);

    // Clear to white background
    display.clear(Rgb888::WHITE)?;

    // Calculate day of week and select quote
    let dow = calculate_day_of_week(time.years, time.months, time.days);
    let doy = day_of_year(time.months, time.days, time.years);
    let quote = select_quote(doy);

    // Choose accent color randomly but consistently for the day
    let accent_colors = [Rgb888::RED, Rgb888::BLUE, Rgb888::GREEN, Rgb888::CSS_ORANGE];
    let accent_color = accent_colors[(doy as usize) % accent_colors.len()];

    // Text styles
    let large_style = MonoTextStyle::new(&FONT_10X20, Rgb888::BLACK);
    let medium_style = MonoTextStyle::new(&FONT_9X18, Rgb888::BLACK);
    let small_style = MonoTextStyle::new(&FONT_6X10, Rgb888::BLACK);
    let accent_style = MonoTextStyle::new(&FONT_10X20, accent_color);

    // Draw decorative top border
    let border_style = PrimitiveStyleBuilder::new()
        .fill_color(accent_color)
        .build();
    Rectangle::new(Point::new(0, 0), Size::new(EPD_7IN3F_WIDTH as u32, 10))
        .into_styled(border_style)
        .draw(display)?;

    // Day of week at top
    let dow_text = day_of_week_name(dow);
    Text::with_alignment(
        dow_text,
        Point::new((EPD_7IN3F_WIDTH / 2) as i32, 40),
        medium_style,
        Alignment::Center,
    )
    .draw(display)?;

    // Large day number (center-ish)
    let mut day_str = heapless::String::<4>::new();
    use core::fmt::Write;
    write!(&mut day_str, "{}", time.days).ok();

    Text::with_alignment(
        &day_str,
        Point::new((EPD_7IN3F_WIDTH / 2) as i32, 120),
        accent_style,
        Alignment::Center,
    )
    .draw(display)?;

    // Month name
    let month_text = month_name(time.months);
    Text::with_alignment(
        month_text,
        Point::new((EPD_7IN3F_WIDTH / 2) as i32, 170),
        large_style,
        Alignment::Center,
    )
    .draw(display)?;

    // Year
    let mut year_str = heapless::String::<8>::new();
    write!(&mut year_str, "{}", time.years).ok();

    Text::with_alignment(
        &year_str,
        Point::new((EPD_7IN3F_WIDTH / 2) as i32, 200),
        medium_style,
        Alignment::Center,
    )
    .draw(display)?;

    // Decorative line
    Line::new(
        Point::new(100, 230),
        Point::new((EPD_7IN3F_WIDTH - 100) as i32, 230),
    )
    .into_styled(PrimitiveStyle::with_stroke(accent_color, 2))
    .draw(display)?;

    // Quote - word wrap it manually for now
    let quote_y_start = 270;
    let line_height = 18;

    // Simple word wrapping
    let words: heapless::Vec<&str, 32> = quote.split_whitespace().collect();
    let mut current_line = heapless::String::<80>::new();
    let mut y_pos = quote_y_start;

    for word in words.iter() {
        let test_line = if current_line.is_empty() {
            let mut s = heapless::String::<80>::new();
            s.push_str(word).ok();
            s
        } else {
            let mut temp = current_line.clone();
            temp.push(' ').ok();
            temp.push_str(word).ok();
            temp
        };

        // Rough estimate: 60 chars per line for FONT_6X10
        if test_line.len() > 60 && !current_line.is_empty() {
            // Draw current line
            Text::with_alignment(
                &current_line,
                Point::new((EPD_7IN3F_WIDTH / 2) as i32, y_pos),
                small_style,
                Alignment::Center,
            )
            .draw(display)?;

            current_line.clear();
            current_line.push_str(word).ok();
            y_pos += line_height;
        } else {
            current_line = test_line;
        }
    }

    // Draw the last line
    if !current_line.is_empty() {
        Text::with_alignment(
            &current_line,
            Point::new((EPD_7IN3F_WIDTH / 2) as i32, y_pos),
            small_style,
            Alignment::Center,
        )
        .draw(display)?;
    }

    Ok(())
}
