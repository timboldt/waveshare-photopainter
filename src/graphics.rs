use embedded_graphics::{
    mono_font::MonoTextStyle,
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
    text::{Alignment, Text},
};
use profont::{PROFONT_18_POINT, PROFONT_24_POINT};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use u8g2_fonts::{types::FontColor, FontRenderer};

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

/// Absurd quotes from historical figures
const QUOTES: [(&str, &str); 61] = [
    ("Sacking Rome is fine. The actual sacking part is great. It's the paperwork afterward that really grinds you down.", "Attila the Hun"),
    ("I have conquered half the known world from the back of a horse, yet I still cannot get my fitted sheet to stay on the corner of the mattress.", "Genghis Khan"),
    ("The key to military strategy is the element of surprise. Which is why I always hide in the kitchen pantry to scare my legionaries when they come in for a snack.", "Julius Caesar"),
    ("An army marches on its stomach. Which is incredibly inconvenient, because we keep tripping over each other. It's a logistical nightmare.", "Napoleon Bonaparte"),
    ("Veni, vidi, wifi. I came, I saw, I asked for the network password.", "Julius Caesar"),
    ("I don't want to conquer the world anymore. I just want to sit down in a quiet room and finally finish a Sudoku without someone trying to assassinate me.", "Alexander the Great"),
    ("People say 'Et tu, Brute?' like it was the ultimate betrayal. Honestly? The real betrayal was when Brute said he'd split an Uber with me and then 'forgot' his wallet.", "Julius Caesar"),
    ("The hardest part about crossing the Alps with elephants wasn't the cold or the terrain. It was the constant stopping for bathroom breaks. We made terrible time.", "Hannibal"),
    ("If I had known history would remember me mostly for being short—even though I was average height for the time—I would have worn much taller hats.", "Napoleon Bonaparte"),
    ("Bathing in asses' milk every day sounds glamorous until you realize how much time you have to spend sourcing that many donkeys in the desert. It's a full-time job.", "Cleopatra"),
    ("Listen, I'm just saying, if you're going to build a Great Wall, at least put in a decent drive-thru window every few miles.", "Genghis Khan"),
    ("It is better to be feared than loved, if you cannot be both. But it is best of all to be the guy who brought the good donuts to the morning meeting.", "Niccolò Machiavelli"),
    ("I can calculate the motion of heavenly bodies, but not the madness of people who stand in the middle of the escalator instead of walking to the right.", "Isaac Newton"),
    ("It is not the strongest of the species that survives, nor the most intelligent. It is the one that remembers to cancel the free trial before the credit card gets charged.", "Charles Darwin"),
    ("E = mc². Energy equals milk coffee squared. It's the only way I can get any work done in this patent office.", "Albert Einstein"),
    ("I didn't actually discover electricity. I just rubbed my socks on the carpet and touched a doorknob, and then things got way out of hand.", "Benjamin Franklin"),
    ("We hold these truths to be self-evident: that all men are created equal, except for people who chew with their mouths open. They are evidently worse.", "Thomas Jefferson"),
    ("Four score and seven years ago... actually, wait. Does anyone have a charger? My phone is at 4% and I have the rest of the speech on there.", "Abraham Lincoln"),
    ("We shall fight on the beaches, we shall fight on the landing grounds... but we shall not fight on the freeway at 5:00 PM on a Friday. We shall sit in traffic and listen to a podcast.", "Winston Churchill"),
    ("I have had six wives. You would think at least one of them would have known how to load the dishwasher correctly. Bowls go on the top rack, Anne!", "Henry VIII"),
    ("We are not amused. We are actually just hungry. Does this palace have any snacks?", "Queen Victoria"),
    ("To be, or not to be? That is the question. The answer is usually 'not to be,' because I cancelled plans to stay home and watch Netflix in my pajamas.", "William Shakespeare"),
    ("I call this masterpiece The Starry Night not because of the celestial beauty, but because I couldn't find my glasses and everything looked kind of blurry.", "Vincent van Gogh"),
    ("Speak softly and carry a big stick. Also, bring a jacket. My mother said it might get chilly later.", "Theodore Roosevelt"),
    ("I can tell you exactly how fast my car was going, Officer. But according to my calculations, that means I have absolutely no idea where it is right now. So technically, I wasn't even here.", "Werner Heisenberg"),
    ("Now I am become Death, the destroyer of worlds. Also, I accidentally put a fork in the microwave again. So, destroyer of appliances, mostly.", "J. Robert Oppenheimer"),
    ("The best part about working with radium isn't the scientific breakthrough. It's that I never stub my toe during a midnight bathroom run. I am my own nightlight.", "Marie Curie"),
    ("God does not play dice with the universe. He plays Monopoly. And just like in Monopoly, He refuses to trade me the Boardwalk property even though I offered Him both utilties.", "Albert Einstein"),
    ("My cat is dead. My cat is alive. Honestly, until I open the box, the only thing that is certain is that I don't have to clean the litter box yet.", "Erwin Schrödinger"),
    ("I am considered the first computer programmer. Which effectively means I was the first person in history to spend six hours staring at a wall, only to realize I forgot a semicolon on line 4.", "Ada Lovelace"),
    ("I cracked the Enigma code in absolute secrecy to save the free world. But for the life of me, I cannot remember which variation of 'Password123!' I used for my Netflix login.", "Alan Turing"),
    ("It is easier to ask forgiveness than it is to get permission. This is especially true when you have just accidentally dropped the production database tables on a Friday afternoon.", "Grace Hopper"),
    ("I envisioned the World Wide Web as a glorious platform for global collaboration and shared human knowledge. I did not anticipate that 90% of it would be videos of cats falling off furniture.", "Tim Berners-Lee"),
    ("I have a dream... that one day, my code will compile without warnings. I have a dream that one day, the documentation will actually match the API.", "Martin Luther King Jr."),
    ("If I have seen further, it is by standing on the shoulders of giants. And by 'giants,' I mean copying and pasting code from Stack Overflow threads from 2013.", "Isaac Newton"),
    ("Let them eat cake. Or brioche. Or honestly, just whatever is in the back of the pantry. I haven't been grocery shopping in weeks and the carriage has a flat tire.", "Marie Antoinette"),
    ("I have the heart and stomach of a king, but the neck of a giraffe who slept in a drafty barn. This collar is ruining my posture.", "Queen Elizabeth I"),
    ("They say I have a reputation for... adventurous romances. But honestly, I just swipe right on everyone because I'm bored and the winter in Russia is very, very long.", "Catherine the Great"),
    ("I am not amusing myself. I am merely trying to get this corset off without dislocating a rib. It is a two-person job, and one of the persons has quit.", "Queen Victoria"),
    ("The most effective way to do it, is to do it. But the second most effective way is to wait until the deadline is 10 minutes away and let the panic fuel you.", "Amelia Earhart"),
    ("I am not afraid of storms, for I am learning how to sail my ship. I am, however, terrified of parallel parking. I will circle the block for an hour to avoid it.", "Louisa May Alcott"),
    ("I'm not saying the voices in my head are right, but they did remind me to turn off the oven before I left the house, so who's the real winner here?", "Joan of Arc"),
    ("I attribute my success to this: I never gave or took an excuse. Also, I washed my hands. Like, constantly. You people are gross.", "Florence Nightingale"),
    ("It is a truth universally acknowledged, that a single man in possession of a good fortune, must be in want of a wife. But it is also a truth that he probably has a weird laugh and replies to texts with just a 'thumbs up' emoji.", "Jane Austen"),
    ("I paint self-portraits because I am so often alone, because I am the person I know best, and because I can't figure out how to turn the camera around on this stupid phone.", "Frida Kahlo"),
    ("Beware; for I am fearless, and therefore powerful. Also, I haven't had my coffee yet, so don't speak to me until noon.", "Mary Shelley"),
    ("The best time to plan a book is while you're doing the dishes. The best time to plan a murder is while you're listening to your neighbor practice the trumpet for the third hour in a row.", "Agatha Christie"),
    ("A wizard is never late, Frodo. Nor is he early. He arrives precisely when... hang on, I put the wrong address in Waze. I'm at a Starbucks five miles away. Start the fellowship without me.", "Gandalf the Grey"),
    ("I find your lack of faith disturbing. I also find the fact that you didn't refill the coffee pot when you took the last cup disturbing. Prepare to die.", "Darth Vader"),
    ("Space: the final frontier. These are the voyages of the Starship Enterprise. Its five-year mission: to find a planet that actually has decent Wi-Fi so I can upload my Captain's Log.", "Captain James T. Kirk"),
    ("One does not simply walk into Mordor. You have to book a reservation months in advance, and the parking is a nightmare. Honestly, it's a tourist trap.", "Boromir"),
    ("Help me, Obi-Wan Kenobi. You're my only hope. I've tried turning the router off and on again, and the blinking light won't stop.", "Princess Leia"),
    ("I am Vengeance. I am the Night. I am... really regretting choosing a costume made entirely of non-breathable rubber. It is incredibly humid in here.", "Batman"),
    ("Elementary, my dear Watson. I knew the suspect was lying not because of his pulse, but because he used the wrong 'your' in his text message. A criminal mind is often grammatically sloppy.", "Sherlock Holmes"),
    ("The name's Bond. James Bond. And I have been trying to reach you about your Aston Martin's extended warranty.", "007"),
    ("With great power comes great responsibility. And with a spandex suit comes a very specific laundry routine. You can't just throw this thing in the dryer.", "Spider-Man"),
    ("It belongs in a museum! Along with that casserole in the back of your fridge! It's been there since the Reagan administration!", "Indiana Jones"),
    ("I'll get you, my pretty, and your little dog too! But first, does anyone have an Ibuprofen? My head is killing me. This green face paint smells like chemicals.", "The Wicked Witch of the West"),
    ("Listen, I don't drink... wine. I drink Red Bull. Do you know how hard it is to stay up all night haunting people? I'm exhausted.", "Count Dracula"),
    ("Frankly, my dear, I don't give a damn. But if you touch the thermostat, we are going to have a serious problem. I like it at 68 degrees.", "Rhett Butler"),
    ("It's alive! IT'S ALIVE! Oh, wait, never mind. I just forgot to unplug the toaster. It's just toast.", "Dr. Frankenstein"),
];

/// Select a quote based on the day of year
pub fn select_quote(day_of_year: u16) -> (&'static str, &'static str) {
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
    let (quote_text, author) = select_quote(doy);

    // Choose accent color randomly but consistently for the day
    let accent_colors = [Rgb888::RED, Rgb888::GREEN];
    let accent_color = accent_colors[(doy as usize) % accent_colors.len()];

    // Text styles - using ProFont for much larger, more readable text
    let large_style = MonoTextStyle::new(&PROFONT_24_POINT, Rgb888::BLACK);
    let medium_style = MonoTextStyle::new(&PROFONT_24_POINT, Rgb888::BLACK);
    let small_style = MonoTextStyle::new(&PROFONT_18_POINT, Rgb888::BLACK);

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
        Point::new((EPD_7IN3F_WIDTH / 2) as i32, 50),
        medium_style,
        Alignment::Center,
    )
    .draw(display)?;

    // Large day number (center-ish) - using huge u8g2 font
    let mut day_str = heapless::String::<4>::new();
    use core::fmt::Write;
    write!(&mut day_str, "{}", time.days).ok();

    // Use u8g2 large number font (62 pixels tall, ~2.5x larger than PROFONT_24_POINT)
    let font_renderer = FontRenderer::new::<u8g2_fonts::fonts::u8g2_font_logisoso62_tn>();

    // Render centered large day number
    font_renderer
        .render_aligned(
            day_str.as_str(),
            Point::new((EPD_7IN3F_WIDTH / 2) as i32, 150),
            u8g2_fonts::types::VerticalPosition::Baseline,
            u8g2_fonts::types::HorizontalAlignment::Center,
            FontColor::Transparent(accent_color),
            display,
        )
        .ok();

    // Month name
    let month_text = month_name(time.months);
    Text::with_alignment(
        month_text,
        Point::new((EPD_7IN3F_WIDTH / 2) as i32, 200),
        large_style,
        Alignment::Center,
    )
    .draw(display)?;

    // Year
    let mut year_str = heapless::String::<8>::new();
    write!(&mut year_str, "{}", time.years).ok();

    Text::with_alignment(
        &year_str,
        Point::new((EPD_7IN3F_WIDTH / 2) as i32, 240),
        medium_style,
        Alignment::Center,
    )
    .draw(display)?;

    // Decorative line
    Line::new(
        Point::new(100, 270),
        Point::new((EPD_7IN3F_WIDTH - 100) as i32, 270),
    )
    .into_styled(PrimitiveStyle::with_stroke(accent_color, 2))
    .draw(display)?;

    // Quote - word wrap it manually for now
    let quote_y_start = 300;
    let line_height = 30;

    // Simple word wrapping for quote text
    let words: heapless::Vec<&str, 32> = quote_text.split_whitespace().collect();
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

        // Rough estimate: 40 chars per line for PROFONT_18_POINT
        if test_line.len() > 40 && !current_line.is_empty() {
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

    // Draw the last line of quote
    if !current_line.is_empty() {
        Text::with_alignment(
            &current_line,
            Point::new((EPD_7IN3F_WIDTH / 2) as i32, y_pos),
            small_style,
            Alignment::Center,
        )
        .draw(display)?;
        y_pos += line_height;
    }

    // Draw author attribution
    let mut author_line = heapless::String::<80>::new();
    author_line.push_str("- ").ok();
    author_line.push_str(author).ok();

    y_pos += 4; // Small gap before author
    Text::with_alignment(
        &author_line,
        Point::new((EPD_7IN3F_WIDTH / 2) as i32, y_pos),
        small_style,
        Alignment::Center,
    )
    .draw(display)?;

    Ok(())
}
