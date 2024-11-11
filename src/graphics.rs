use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
};
use rand::{rngs::SmallRng, Rng, SeedableRng};

use crate::epaper::{DisplayBuffer, EPD_7IN3F_HEIGHT, EPD_7IN3F_WIDTH};

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
