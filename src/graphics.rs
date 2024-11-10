
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{
        line, Circle, CornerRadii, Ellipse, Line, PrimitiveStyle, Rectangle, RoundedRectangle,
        Triangle,
    },
};
use embedded_graphics_simulator::{OutputSettings, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window};
use rand::Rng;

fn update(display: &mut SimulatorDisplay<Rgb888>) -> Result<(), core::convert::Infallible> {
    display.clear(Rgb888::BLACK)?;

    for color in [
        Rgb888::CSS_ORANGE_RED,
        Rgb888::CSS_GOLD,
        Rgb888::CSS_SEA_GREEN,
        Rgb888::CSS_TEAL,
        Rgb888::CSS_STEEL_BLUE,
        Rgb888::CSS_FUCHSIA,
    ] {
        let line_style = PrimitiveStyle::with_stroke(color, 3);
        let x = rand::thread_rng().gen_range(100..700);
        let y = rand::thread_rng().gen_range(100..380);
        let mut p = Point::new(x, y);
        for _ in 0..2000 {
            let prev_p = p;
            let r = rand::thread_rng().gen_range(0..4);
            let step_size = 5;
            match r {
                0 => p.x += step_size,
                1 => p.x -= step_size,
                2 => p.y += step_size,
                3 => p.y -= step_size,
                _ => (),
            }
            Line::new(prev_p, p).into_styled(line_style).draw(display)?;
        }
    }

    Ok(())
}