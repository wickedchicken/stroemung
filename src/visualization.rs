use crate::cell::Cell;
use crate::math::Real;
use crate::simulation::Simulation;
use macroquad::prelude::Color;
use macroquad::prelude::Image;

fn hsl_to_rgb(hue: f32, saturation: f32, lightness: f32) -> (f32, f32, f32) {
    let c = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
    let m = lightness - c / 2.0;

    let (r, g, b) = if hue < 60.0 {
        (c, x, 0.0)
    } else if hue < 120.0 {
        (x, c, 0.0)
    } else if hue < 180.0 {
        (0.0, c, x)
    } else if hue < 240.0 {
        (0.0, x, c)
    } else if hue < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (r + m, g + m, b + m)
}

fn color_speed(cell_type: Cell, u: Real, v: Real, speed_range: [Real; 2]) -> Color {
    match cell_type {
        Cell::Fluid => {
            let speed = (u.powi(2) + v.powi(2)).sqrt();

            // 240 offset to map from blue to red instead of the whole range of hue
            let hue: f32 = (240.0
                - (speed - speed_range[0]) * 240.0 / (speed_range[1] - speed_range[0]))
                as f32;
            let saturation = 1.0;
            let lightness = 0.5;

            let (r, g, b) = hsl_to_rgb(hue, saturation, lightness);

            Color::new(r, g, b, 1.0)
        }
        Cell::Boundary(_) => Color::new(0.5, 0.5, 0.5, 1.0),
    }
}

fn color_pressure(cell_type: Cell, pressure: Real, pressure_range: [f64; 2]) -> Color {
    match cell_type {
        Cell::Fluid => {
            // 240 offset to map from blue to red instead of the whole range of hue
            let offset = 240.0;
            let hue: f32 = (offset
                - (pressure - pressure_range[0]) * offset
                    / (pressure_range[1] - pressure_range[0]))
                as f32;
            let saturation = 1.0;
            let lightness = 0.5;

            let (r, g, b) = hsl_to_rgb(hue, saturation, lightness);
            Color::new(r, g, b, 1.0)

            // let value = 1.0 - ((cell.pressure - pressure_range[0]) / (pressure_range[1] - pressure_range[0]));
            //
            // Color::new(value, value, value, 1.0)
        }
        Cell::Boundary(_) => Color::new(0.5, 0.0, 0.0, 1.0),
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ColorType {
    #[default]
    Pressure,
    Speed,
}

pub fn render_simulation(
    simulation: &Simulation,
    image: &mut Image,
    w: usize,
    h: usize,
    color_type: ColorType,
) {
    for x in 0..w {
        for y in 0..h {
            let cell_type = simulation.grid.cell_type[(x, y)];
            let color = match color_type {
                ColorType::Pressure => color_pressure(
                    cell_type,
                    simulation.grid.pressure[(x, y)],
                    simulation.grid.pressure_range,
                ),
                ColorType::Speed => color_speed(
                    cell_type,
                    simulation.grid.u[(x, y)],
                    simulation.grid.v[(x, y)],
                    simulation.grid.speed_range,
                ),
            };
            image.set_pixel(x as u32, y as u32, color);
        }
    }
}
