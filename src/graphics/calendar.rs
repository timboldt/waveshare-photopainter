use embedded_graphics::{
    mono_font::MonoTextStyle,
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
    text::{Alignment, Text},
};
use profont::{PROFONT_18_POINT, PROFONT_24_POINT};
use rand::{rngs::SmallRng, SeedableRng};
use u8g2_fonts::{types::FontColor, FontRenderer};

use crate::{
    epaper::{DisplayBuffer, EPD_7IN3F_WIDTH},
    graphics::ltree,
    rtc::TimeData,
};

// Calendar layout constants
const CONTENT_CENTER_X: i32 = EPD_7IN3F_WIDTH as i32 / 2;
const BORDER_MARGIN: i32 = 100;
const DECORATIVE_LINE_Y: i32 = 270;
const QUOTE_START_Y: i32 = 300;
const LINE_HEIGHT: i32 = 30;
const MAX_CHARS_PER_LINE: usize = 40;

/// Inspirational quotes from quotable.io
const QUOTES: [(&str, &str); 141] = [
    ("In the depth of winter, I finally learned that there was within me an invincible summer.", "Albert Camus"),
    ("To follow, without halt, one aim: There is the secret of success.", "Anna Pavlova"),
    ("It is never too late to be what you might have been.", "George Eliot"),
    ("Pick battles big enough to matter, small enough to win.", "Jonathan Kozol"),
    ("Who sows virtue reaps honor.", "Leonardo da Vinci"),
    ("Nobody will believe in you unless you believe in yourself.", "Liberace"),
    ("Each day provides its own gifts.", "Marcus Aurelius"),
    ("Don't wait. The time will never be just right.", "Napoleon Hill"),
    ("Do not go where the path may lead, go instead where there is no path and leave a trail.", "Ralph Waldo Emerson"),
    ("Nothing great was ever achieved without enthusiasm.", "Ralph Waldo Emerson"),
    ("If you love someone, set them free.", "Richard Bach"),
    ("There is no duty we so underrate as the duty of being happy.", "Robert Louis Stevenson"),
    ("These days people seek knowledge, not wisdom.", "Vernon Cooper"),
    ("Short words are best and the old words when short are best of all.", "Winston Churchill"),
    ("Applause is a receipt, not a bill.", "Dale Carnegie"),
    ("Never interrupt someone doing what you said couldn't be done.", "Amelia Earhart"),
    ("Some of the best lessons we ever learn are learned from past mistakes.", "Dale Turner"),
    ("Every friendship goes through ups and downs.", "Mariella Frostrup"),
    ("Do something wonderful, people may imitate it.", "Albert Schweitzer"),
    ("As you think, so shall you become.", "Bruce Lee"),
    ("The universe is full of magical things, patiently waiting for our wits to grow sharper.", "Eden Phillpotts"),
    ("Those who cannot learn from history are doomed to repeat it.", "George Santayana"),
    ("Every adversity carries with it the seed of an equal or greater benefit.", "Napoleon Hill"),
    ("Well done is better than well said.", "Benjamin Franklin"),
    ("We must not allow ourselves to become like the system we oppose.", "Desmond Tutu"),
    ("Do not give your attention to what others do; give it to what you do.", "Dhammapada"),
    ("People grow through experience if they meet life honestly and courageously.", "Eleanor Roosevelt"),
    ("If you wish to be a writer, write.", "Epictetus"),
    ("But man is not made for defeat.", "Ernest Hemingway"),
    ("You can't shake hands with a clenched fist.", "Indira Gandhi"),
    ("You can't stop the waves, but you can learn to surf.", "Jon Kabat-Zinn"),
    ("If you correct your mind, the rest of your life will fall into place.", "Laozi"),
    ("Imagination is the highest kite one can fly.", "Lauren Bacall"),
    ("I never see what has been done; I only see what remains to be done.", "Marie Curie"),
    ("No man can succeed in a line of endeavor which he does not like.", "Napoleon Hill"),
    ("By believing passionately in something that does not yet exist, we create it.", "Nikos Kazantzakis"),
    ("The meaning I picked, the one that changed my life: Overcome fear, behold wonder.", "Richard Bach"),
    ("It's so simple to be wise. Just think of something stupid to say and then don't.", "Sam Levenson"),
    ("Everything you are against weakens you. Everything you are for empowers you.", "Wayne Dyer"),
    ("Do more than dream: work.", "William Arthur Ward"),
    ("The function of wisdom is to discriminate between good and evil.", "Cicero"),
    ("Genius unrefined resembles a flash of lightning, but wisdom is like the sun.", "Franz Grillparzer"),
    ("The greatest healing therapy is friendship and love.", "Hubert Humphrey"),
    ("Those that know, do. Those that understand, teach.", "Aristotle"),
    ("We need to find the courage to say NO to things not serving us.", "Barbara De Angelis"),
    ("We must learn our limits. We are all something, but none of us are everything.", "Blaise Pascal"),
    ("Choose a job you love, and you will never have to work a day in your life.", "Confucius"),
    ("They must often change, who would be constant in happiness or wisdom.", "Confucius"),
    ("Respect should be earned by actions, and not acquired by years.", "Frank Lloyd Wright"),
    ("We are all inclined to judge ourselves by our ideals; others, by their acts.", "Harold Nicolson"),
    ("Correction does much, but encouragement does more.", "Johann Wolfgang von Goethe"),
    ("All the great performers I have worked with are fueled by a personal dream.", "John Eliot"),
    ("Beauty is not in the face; beauty is a light in the heart.", "Kahlil Gibran"),
    ("You were not born a winner, and you were not born a loser.", "Lou Holtz"),
    ("What we achieve inwardly will change outer reality.", "Plutarch"),
    ("Our strength grows out of our weaknesses.", "Ralph Waldo Emerson"),
    ("Things that were hard to bear are sweet to remember.", "Seneca the Younger"),
    ("The supreme art of war is to subdue the enemy without fighting.", "Sun Tzu"),
    ("When people are like each other they tend to like each other.", "Tony Robbins"),
    ("You have enemies? Good. That means you've stood up for something.", "Winston Churchill"),
    ("When fate hands us a lemon, let's try to make lemonade.", "Dale Carnegie"),
    ("One must be fond of people and trust them if one is not to make a mess of life.", "E. M. Forster"),
    ("All serious daring starts from within.", "Harriet Beecher Stowe"),
    ("Trouble is only opportunity in work clothes.", "Henry J. Kaiser"),
    ("Love is the flower you've got to let grow.", "John Lennon"),
    ("Quality is never an accident; it is always the result of intelligent effort.", "John Ruskin"),
    ("We shall never know all the good that a simple smile can do.", "Mother Teresa"),
    ("Fear not for the future, weep not for the past.", "Percy Bysshe Shelley"),
    ("Memory is the mother of all wisdom.", "Samuel Johnson"),
    ("Life is the flower for which love is the honey.", "Victor Hugo"),
    ("Courage is what it takes to stand up and speak; courage is also what it takes to sit down and listen.", "Winston Churchill"),
    ("Never, never, never give up.", "Winston Churchill"),
    ("The key is to keep company only with people who uplift you.", "Epictetus"),
    ("It's the little details that are vital. Little things make big things happen.", "John Wooden"),
    ("If you don't know where you are going, any road will get you there.", "Lewis Carroll"),
    ("Habit, if not resisted, soon becomes necessity.", "Augustine of Hippo"),
    ("However rare true love may be, it is less so than true friendship.", "François de La Rochefoucauld"),
    ("True friendship is like sound health; the value of it is seldom known until it is lost.", "Charles Caleb Colton"),
    ("What you do not want done to yourself, do not do to others.", "Confucius"),
    ("When you see a man of worth, think of how you may emulate him.", "Confucius"),
    ("A successful person is one who can lay a firm foundation with the bricks others throw.", "David Brinkley"),
    ("You must do the things you think you cannot do.", "Eleanor Roosevelt"),
    ("Three things in human life are important: The first is to be kind.", "Henry James"),
    ("Be less curious about people and more curious about ideas.", "Marie Curie"),
    ("Happiness is as a butterfly which, when pursued, is always beyond our grasp.", "Nathaniel Hawthorne"),
    ("By nature, man hates change; seldom will he quit his old home till it has fallen.", "Thomas Carlyle"),
    ("Minds are like parachutes. They only function when open.", "Thomas Dewar"),
    ("I can't imagine a person becoming a success who doesn't give this game everything.", "Walter Cronkite"),
    ("Adopt the pace of nature: her secret is patience.", "Ralph Waldo Emerson"),
    ("The only true wisdom is in knowing you know nothing.", "Isocrates"),
    ("Nature and books belong to the eyes that see them.", "Ralph Waldo Emerson"),
    ("Peace cannot be kept by force. It can only be achieved by understanding.", "Albert Einstein"),
    ("If you can't explain it simply, you don't understand it well enough.", "Albert Einstein"),
    ("To have much learning and skill, to be well-trained in discipline, is the highest blessing.", "The Buddha"),
    ("A single lamp may light hundreds of thousands of lamps without itself being diminished.", "The Buddha"),
    ("How many cares one loses when one decides not to be something but to be someone.", "Coco Chanel"),
    ("I cannot give you the formula for success, but I can give you the formula for failure.", "Herbert Bayard Swope"),
    ("There are two kinds of failures: those who thought and never did.", "Laurence J. Peter"),
    ("Do not be too timid and squeamish about your reactions. All life is an experiment.", "Ralph Waldo Emerson"),
    ("Only those who dare to fail greatly can ever achieve greatly.", "Robert F. Kennedy"),
    ("Change your life today. Don't gamble on the future, act now, without delay.", "Simone de Beauvoir"),
    ("Independence is happiness.", "Susan B. Anthony"),
    ("There is nothing on this earth more to be prized than true friendship.", "Thomas Aquinas"),
    ("What we think determines what happens to us.", "Wayne Dyer"),
    ("Look up at the stars and not down at your feet. Try to make sense of what you see.", "Stephen Hawking"),
    ("The doors of wisdom are never shut.", "Benjamin Franklin"),
    ("The strong bond of friendship is not always a balanced equation.", "Simon Sinek"),
    ("Rare as is true love, true friendship is rarer.", "Jean de La Fontaine"),
    ("Blessed is the man who expects nothing, for he shall never be disappointed.", "Alexander Pope"),
    ("Kind words do not cost much. Yet they accomplish much.", "Blaise Pascal"),
    ("You can only grow if you're willing to feel awkward and uncomfortable.", "Brian Tracy"),
    ("It does not matter how slowly you go as long as you do not stop.", "Confucius"),
    ("Fine words and an insinuating appearance are seldom associated with true virtue.", "Confucius"),
    ("The superior man acts before he speaks.", "Confucius"),
    ("Things do not change; we change.", "Henry David Thoreau"),
    ("Very little is needed to make a happy life; it is all within yourself.", "Marcus Aurelius"),
    ("If you want a thing done well, do it yourself.", "Napoleon"),
    ("The only journey is the one within.", "Rainer Maria Rilke"),
    ("Good thoughts are no better than good dreams, unless they be executed.", "Ralph Waldo Emerson"),
    ("Time changes everything except something within us which is always surprised by change.", "Thomas Hardy"),
    ("The way we communicate with others ultimately determines the quality of our lives.", "Tony Robbins"),
    ("I know where I'm going and I know the truth, and I don't have to be what you want me to be.", "Muhammad Ali"),
    ("Between saying and doing, many a pair of shoes is worn out.", "Iris Murdoch"),
    ("True happiness arises from the enjoyment of oneself and friendship of select companions.", "Joseph Addison"),
    ("Our most intimate friend is not he to whom we show the worst.", "Nathaniel Hawthorne"),
    ("There is no friendship, no love, like that of the parent for the child.", "Henry Ward Beecher"),
    ("Always bear in mind that your own resolution to succeed is more important.", "Abraham Lincoln"),
    ("Once we accept our limits, we go beyond them.", "Albert Einstein"),
    ("Feeling and longing are the motive forces behind all human endeavor.", "Albert Einstein"),
    ("We all live with the objective of being happy; our lives are all different.", "Anne Frank"),
    ("There is only one success: to be able to spend your life in your own way.", "Christopher Morley"),
    ("When deeds and words are in accord, the whole world is transformed.", "Zhuang Zhou"),
    ("Until you make peace with who you are, you'll never be content with what you have.", "Doris Mortman"),
    ("Do what you can. Want what you have. Be who you are.", "Forrest Church"),
    ("If you think you can, you can. And if you think you can't, you're right.", "Henry Ford"),
    ("Silence is a source of great strength.", "Laozi"),
    ("All difficult things have their origin in that which is easy.", "Laozi"),
    ("Always be smarter than the people who hire you.", "Lena Horne"),
    ("Although there may be tragedy in your life, there's always a possibility to triumph.", "Oprah Winfrey"),
    ("Most of the shadows of life are caused by standing in our own sunshine.", "Ralph Waldo Emerson"),
    ("The mark of your ignorance is the depth of your belief in injustice and tragedy.", "Richard Bach"),
];

/// Select a quote based on the day of year
fn select_quote(day_of_year: u16) -> (&'static str, &'static str) {
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
    let mut rng = SmallRng::seed_from_u64(seed);

    // Clear to white background
    display.clear(Rgb888::WHITE)?;

    // Calculate day of week and select quote
    let dow = calculate_day_of_week(time.years, time.months, time.days);
    let doy = day_of_year(time.months, time.days, time.years);
    let (quote_text, author) = select_quote(doy);

    // Choose accent color randomly but consistently for the day
    let accent_colors = [Rgb888::RED, Rgb888::GREEN, Rgb888::BLUE];
    let color_index = (doy as usize) % accent_colors.len();
    let accent_color = accent_colors[color_index];

    // Text styles - using ProFont for much larger, more readable text
    let large_style = MonoTextStyle::new(&PROFONT_24_POINT, Rgb888::BLACK);
    let medium_style = MonoTextStyle::new(&PROFONT_24_POINT, Rgb888::BLACK);
    let small_style = MonoTextStyle::new(&PROFONT_18_POINT, Rgb888::BLACK);

    // L-system pattern configurations
    // Match on color_index since Rgb888 struct matching doesn't work reliably
    // Index: 0 = RED, 1 = GREEN, 2 = BLUE
    let (axiom, rules, angle, min_iter, max_iter, step_len): (
        &str,
        &[(&str, &str)],
        f32,
        usize,
        usize,
        f32,
    ) = match color_index {
        0 => {
            // RED: The Peony (Closed Gosper Island)
            // We repeat the base XF sequence 6 times with a turn (-) to close the loop into a flower.
            (
                "XF-XF-XF-XF-XF-XF",
                &[
                    ("X", "X+YF++YF-FX--FXFX-YF+"),
                    ("Y", "-FX+YFYF++YF+FX--FX+Y"),
                ],
                60.0,
                3,
                3,   // MAX 3. Iteration 4 is too heavy for embedded.
                2.0, // Keep step size large (2.0 - 4.0)
            )
        }
        1 => {
            // Green: The L-system tree
            // Instead, we increase step_len and rely on the structure.
            (
                "X",
                &[("X", "F-[[X]+X]+F[+FX]-X"), ("F", "FF")],
                22.5,
                5,
                5,   // Lowered iterations (5 is huge with FF expansion)
                2.0, // Minimum visible line width
            )
        }
        _ => {
            // Geometric Rose (Blue)
            // This is a "Koch Snowflake" variant.
            (
                "F++F++F++F++F++F",
                &[("F", "F-F++F-F")], // Standard Snowflake rule (cleaner than the pipe rule)
                60.0,
                3,
                3,
                3.0, // Needs distinct lines to look like crystals
            )
        }
    };

    // Top-left corner
    ltree::draw_ltree(
        display,
        accent_color,
        100,
        150,
        axiom,
        rules,
        angle,
        min_iter,
        max_iter,
        step_len,
        &mut rng,
    )
    .ok();

    // Bottom-right corner
    ltree::draw_ltree(
        display,
        accent_color,
        EPD_7IN3F_WIDTH as i32 - 100,
        150,
        axiom,
        rules,
        angle,
        min_iter,
        max_iter,
        step_len,
        &mut rng,
    )
    .ok();

    // Day of week at top
    let dow_text = day_of_week_name(dow);
    Text::with_alignment(
        dow_text,
        Point::new(CONTENT_CENTER_X, 50),
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
            Point::new(CONTENT_CENTER_X, 150),
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
        Point::new(CONTENT_CENTER_X, 200),
        large_style,
        Alignment::Center,
    )
    .draw(display)?;

    // Year
    let mut year_str = heapless::String::<8>::new();
    write!(&mut year_str, "{}", time.years).ok();

    Text::with_alignment(
        &year_str,
        Point::new(CONTENT_CENTER_X, 240),
        medium_style,
        Alignment::Center,
    )
    .draw(display)?;

    // Decorative line (with margins on both sides)
    Line::new(
        Point::new(BORDER_MARGIN, DECORATIVE_LINE_Y),
        Point::new(EPD_7IN3F_WIDTH as i32 - BORDER_MARGIN, DECORATIVE_LINE_Y),
    )
    .into_styled(PrimitiveStyle::with_stroke(accent_color, 2))
    .draw(display)?;

    // Quote - word wrap it manually for now
    let quote_y_start = QUOTE_START_Y;
    let line_height = LINE_HEIGHT;

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

        // Rough estimate: MAX_CHARS_PER_LINE for PROFONT_18_POINT
        if test_line.len() > MAX_CHARS_PER_LINE && !current_line.is_empty() {
            // Draw current line
            Text::with_alignment(
                &current_line,
                Point::new(CONTENT_CENTER_X, y_pos),
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
            Point::new(CONTENT_CENTER_X, y_pos),
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
        Point::new(CONTENT_CENTER_X, y_pos),
        small_style,
        Alignment::Center,
    )
    .draw(display)?;

    Ok(())
}
