//! L-system fractal renderer
//!
//! Implements a recursive (streaming) L-system interpreter.
//! It does not buffer the string; it calculates geometry on the fly.
//! Includes a measurement pass to center fractals on their starting point.

use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
};
use micromath::F32Ext;

use crate::epaper::DisplayBuffer;

/// Configuration
const TURTLE_STACK_SIZE: usize = 128; // Depth of branching logic ( '[' )

/// Predefined L-system patterns
pub struct LSystemPattern {
    pub axiom: &'static str,
    pub rules: &'static [(&'static str, &'static str)],
    pub angle: f32,
    pub iterations: usize,
    pub step_length: f32,
}

/// Red: Gosper curve arranged as a 6-fold flower (Peony)
pub const PATTERN_PEONY: LSystemPattern = LSystemPattern {
    axiom: "XF-XF-XF-XF-XF-XF",
    rules: &[
        ("X", "X+YF++YF-FX--FXFX-YF+"),
        ("Y", "-FX+YFYF++YF+FX--FX+Y"),
    ],
    angle: 60.0,
    iterations: 3,
    step_length: 2.0,
};

/// Green: Classic L-system tree (vine)
pub const PATTERN_TREE: LSystemPattern = LSystemPattern {
    axiom: "X",
    rules: &[("X", "F-[[X]+X]+F[+FX]-X"), ("F", "FF")],
    angle: 22.5,
    iterations: 5,
    step_length: 2.0,
};

/// Blue: Koch snowflake variant (geometric rose)
pub const PATTERN_SNOWFLAKE: LSystemPattern = LSystemPattern {
    axiom: "F++F++F++F++F++F",
    rules: &[("F", "F-F++F-F")],
    angle: 60.0,
    iterations: 3,
    step_length: 3.0,
};

/// Represents the state of the drawing cursor
#[derive(Clone, Copy)]
struct TurtleState {
    x: f32,
    y: f32,
    angle: f32,
}

/// Bounding box tracker for measurement pass
#[derive(Clone, Copy)]
struct BoundingBox {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

impl BoundingBox {
    fn new() -> Self {
        Self {
            min_x: 0.0,
            max_x: 0.0,
            min_y: 0.0,
            max_y: 0.0,
        }
    }

    fn update(&mut self, x: f32, y: f32) {
        if x < self.min_x {
            self.min_x = x;
        }
        if x > self.max_x {
            self.max_x = x;
        }
        if y < self.min_y {
            self.min_y = y;
        }
        if y > self.max_y {
            self.max_y = y;
        }
    }

    fn center(&self) -> (f32, f32) {
        (
            (self.min_x + self.max_x) / 2.0,
            (self.min_y + self.max_y) / 2.0,
        )
    }
}

/// Measurement turtle - tracks bounding box without drawing
struct MeasureTurtle {
    state: TurtleState,
    step_length: f32,
    turn_angle: f32,
    stack: heapless::Vec<TurtleState, TURTLE_STACK_SIZE>,
    bounds: BoundingBox,
}

impl MeasureTurtle {
    fn new(step_length: f32, angle_degrees: f32) -> Self {
        Self {
            state: TurtleState {
                x: 0.0,
                y: 0.0,
                angle: -core::f32::consts::FRAC_PI_2, // Start pointing UP
            },
            step_length,
            turn_angle: angle_degrees.to_radians(),
            stack: heapless::Vec::new(),
            bounds: BoundingBox::new(),
        }
    }

    fn execute_command(&mut self, command: char) {
        match command {
            'F' => {
                let (sin, cos) = self.state.angle.sin_cos();
                self.state.x += cos * self.step_length;
                self.state.y += sin * self.step_length;
                self.bounds.update(self.state.x, self.state.y);
            }
            '+' => self.state.angle += self.turn_angle,
            '-' => self.state.angle -= self.turn_angle,
            '|' => self.state.angle += core::f32::consts::PI,
            '[' => {
                self.stack.push(self.state).ok();
            }
            ']' => {
                if let Some(s) = self.stack.pop() {
                    self.state = s;
                }
            }
            _ => {}
        }
    }
}

/// Drawing turtle - actually renders to display
struct DrawTurtle<'a> {
    display: &'a mut DisplayBuffer,
    color: Rgb888,
    state: TurtleState,
    step_length: f32,
    turn_angle: f32,
    stack: heapless::Vec<TurtleState, TURTLE_STACK_SIZE>,
}

impl<'a> DrawTurtle<'a> {
    fn new(
        display: &'a mut DisplayBuffer,
        color: Rgb888,
        x: f32,
        y: f32,
        step_length: f32,
        angle_degrees: f32,
    ) -> Self {
        Self {
            display,
            color,
            state: TurtleState {
                x,
                y,
                angle: -core::f32::consts::FRAC_PI_2, // Start pointing UP
            },
            step_length,
            turn_angle: angle_degrees.to_radians(),
            stack: heapless::Vec::new(),
        }
    }

    fn execute_command(&mut self, command: char) {
        match command {
            'F' => {
                let (sin, cos) = self.state.angle.sin_cos();
                let new_x = self.state.x + cos * self.step_length;
                let new_y = self.state.y + sin * self.step_length;

                let start = Point::new(self.state.x as i32, self.state.y as i32);
                let end = Point::new(new_x as i32, new_y as i32);

                Line::new(start, end)
                    .into_styled(PrimitiveStyle::with_stroke(self.color, 1))
                    .draw(self.display)
                    .ok();

                self.state.x = new_x;
                self.state.y = new_y;
            }
            '+' => self.state.angle += self.turn_angle,
            '-' => self.state.angle -= self.turn_angle,
            '|' => self.state.angle += core::f32::consts::PI,
            '[' => {
                self.stack.push(self.state).ok();
            }
            ']' => {
                if let Some(s) = self.stack.pop() {
                    self.state = s;
                }
            }
            _ => {}
        }
    }
}

/// Recursive engine for measurement pass
fn measure_recursive(
    turtle: &mut MeasureTurtle,
    sequence: &str,
    rules: &[(&str, &str)],
    depth: usize,
) {
    for ch in sequence.chars() {
        let mut expanded = false;

        if depth > 0 {
            for (key, replacement) in rules {
                if key.len() == 1 && key.starts_with(ch) {
                    measure_recursive(turtle, replacement, rules, depth - 1);
                    expanded = true;
                    break;
                }
            }
        }

        if !expanded {
            turtle.execute_command(ch);
        }
    }
}

/// Recursive engine for drawing pass
fn draw_recursive(turtle: &mut DrawTurtle, sequence: &str, rules: &[(&str, &str)], depth: usize) {
    for ch in sequence.chars() {
        let mut expanded = false;

        if depth > 0 {
            for (key, replacement) in rules {
                if key.len() == 1 && key.starts_with(ch) {
                    draw_recursive(turtle, replacement, rules, depth - 1);
                    expanded = true;
                    break;
                }
            }
        }

        if !expanded {
            turtle.execute_command(ch);
        }
    }
}

/// Draw an L-system pattern centered at the given position
pub fn draw_ltree(
    display: &mut DisplayBuffer,
    color: Rgb888,
    center_x: i32,
    center_y: i32,
    pattern: &LSystemPattern,
) -> Result<(), core::convert::Infallible> {
    // === PASS 1: Measurement ===
    // Run the L-system starting at (0, 0) to find the bounding box
    let mut measure_turtle = MeasureTurtle::new(pattern.step_length, pattern.angle);
    measure_recursive(&mut measure_turtle, pattern.axiom, pattern.rules, pattern.iterations);

    // Calculate the center of the bounding box
    let (bbox_center_x, bbox_center_y) = measure_turtle.bounds.center();

    // Calculate adjusted starting position to center the fractal
    let adjusted_x = center_x as f32 - bbox_center_x;
    let adjusted_y = center_y as f32 - bbox_center_y;

    // === PASS 2: Drawing ===
    // Now draw with the adjusted starting position
    let mut draw_turtle = DrawTurtle::new(
        display,
        color,
        adjusted_x,
        adjusted_y,
        pattern.step_length,
        pattern.angle,
    );
    draw_recursive(&mut draw_turtle, pattern.axiom, pattern.rules, pattern.iterations);

    Ok(())
}
