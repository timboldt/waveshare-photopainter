//! L-system tree generation
//!
//! Uses a simple L-system to generate a fractal tree pattern.
//! The tree grows upward in the left bar area of the calendar.

use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
};
use micromath::F32Ext;
use rand::{rngs::SmallRng, Rng, SeedableRng};

use crate::epaper::DisplayBuffer;

/// L-system parameters
const MAX_LSYSTEM_LENGTH: usize = 2048;

/// Generate L-system string
fn generate_lsystem(
    axiom: &str,
    rules: &[(&str, &str)],
    iterations: usize,
) -> heapless::String<MAX_LSYSTEM_LENGTH> {
    let mut current = heapless::String::<MAX_LSYSTEM_LENGTH>::new();
    current.push_str(axiom).ok();

    for _ in 0..iterations {
        let mut next = heapless::String::<MAX_LSYSTEM_LENGTH>::new();

        for ch in current.chars() {
            let mut matched = false;
            for (pattern, replacement) in rules {
                if pattern.len() == 1 && pattern.chars().next().unwrap() == ch {
                    next.push_str(replacement).ok();
                    matched = true;
                    break;
                }
            }
            if !matched {
                next.push(ch).ok();
            }
        }

        current = next;
    }

    current
}

/// Draw L-system tree in the left bar area
pub fn draw_ltree(
    display: &mut DisplayBuffer,
    color: Rgb888,
    bar_width: u32,
    display_height: u32,
    seed: u64,
) -> Result<(), core::convert::Infallible> {
    let mut rng = SmallRng::seed_from_u64(seed);

    // L-system rules for a simple tree
    // F = draw forward
    // + = turn right
    // - = turn left
    // [ = push state
    // ] = pop state
    let axiom = "F";
    let rules = [("F", "FF+[+F-F-F]-[-F+F+F]")];

    // Generate L-system with N to N+1 iterations
    let iterations = 6 + (rng.gen_range(0..2));
    let lsystem = generate_lsystem(axiom, &rules, iterations);

    // Drawing parameters
    let angle = 25.0_f32.to_radians(); // Branch angle
    let step_length = 10.0; // Length of each forward step (much longer for bigger tree)

    // Start position (bottom center of the bar)
    let start_x = (bar_width / 3) as f32;
    let start_y = display_height as f32 - 100.0;

    // Initial direction (pointing up)
    let mut x = start_x;
    let mut y = start_y;
    let mut direction = -core::f32::consts::FRAC_PI_2; // -90 degrees (up)

    // Stack for saving/restoring position and direction
    let mut stack: heapless::Vec<(f32, f32, f32), 32> = heapless::Vec::new();

    let line_style = PrimitiveStyle::with_stroke(color, 1);

    // Interpret L-system string
    for ch in lsystem.chars() {
        match ch {
            'F' => {
                // Draw forward
                let new_x = x + direction.cos() * step_length;
                let new_y = y + direction.sin() * step_length;

                // Only draw if within bounds
                if new_x >= 0.0
                    && new_x < bar_width as f32
                    && new_y >= 0.0
                    && new_y < display_height as f32
                {
                    Line::new(
                        Point::new(x as i32, y as i32),
                        Point::new(new_x as i32, new_y as i32),
                    )
                    .into_styled(line_style)
                    .draw(display)
                    .ok();
                }

                x = new_x;
                y = new_y;
            }
            '+' => {
                // Turn right
                direction += angle;
            }
            '-' => {
                // Turn left
                direction -= angle;
            }
            '[' => {
                // Push state
                stack.push((x, y, direction)).ok();
            }
            ']' => {
                // Pop state
                if let Some((saved_x, saved_y, saved_dir)) = stack.pop() {
                    x = saved_x;
                    y = saved_y;
                    direction = saved_dir;
                }
            }
            _ => {
                // Ignore unknown characters
            }
        }
    }

    Ok(())
}
