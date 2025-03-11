pub mod args;
pub mod cell;
pub mod grid;
pub mod math;
pub mod simulation;
pub mod types;
pub mod ui_state;
pub mod visualization;

use crate::ui_state::{initialize_state, MouseState, Preset};
use crate::visualization::render_simulation;
use crate::visualization::ColorType;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use args::Args;
use cell::{BoundaryCell, Cell};
use grid::{presets, SimulationGrid, UnfinalizedSimulationGrid};
use math::Real;
use simulation::{Simulation, UnfinalizedSimulation};
use strum::VariantNames;
use types::GridIndex;

use macroquad::prelude::*;

use macroquad::ui::{hash, root_ui};

pub fn window_conf() -> Conf {
    Conf {
        window_title: "Stroemung".to_owned(),
        ..Default::default()
    }
}

// Draw a 2x2 square since the simulation doesn't support boundary cells that
// have fluid cells on opposite sides.
fn draw_cells(grid: &mut SimulationGrid, cell_type: Cell, m_x: usize, m_y: usize) {
    let mut backup: Vec<(GridIndex, Real, Real, Real, Cell)> = Vec::new();
    let mut modified = false;

    for (x, y) in [
        (m_x, m_y),
        (m_x + 1, m_y),
        (m_x, m_y + 1),
        (m_x + 1, m_y + 1),
    ] {
        // Don't touch outer boundary cells
        if (x > 0) && (x < grid.size[0] - 1) && (y > 0) && (y < grid.size[1] - 1) {
            let idx = (x, y);
            if grid.cell_type[idx] != cell_type {
                // Backup the values so we can restore them in the event that
                // this creates an invalid boundary.
                backup.push((
                    idx,
                    grid.u[idx],
                    grid.v[idx],
                    grid.pressure[idx],
                    grid.cell_type[idx],
                ));
                grid.u[idx] = 0.0;
                grid.v[idx] = 0.0;
                grid.pressure[idx] = 0.0;
                grid.cell_type[idx] = cell_type;
                modified = true;
            }
        }
    }

    if modified && grid.rebuild_boundary_list().is_err() {
        for (idx, u, v, pressure, cell) in backup {
            grid.u[idx] = u;
            grid.v[idx] = v;
            grid.pressure[idx] = pressure;
            grid.cell_type[idx] = cell;
        }
    }
}

fn get_sim(args: &Args, preset: Preset) -> Simulation {
    match &args.sim_file {
        Some(filename) => {
            let file = File::open(Path::new(&filename)).unwrap();
            Simulation::from_reader(BufReader::new(file)).unwrap()
        }
        _ => {
            let size = [args.x_cells, args.y_cells];
            let grid: UnfinalizedSimulationGrid = match preset {
                Preset::Obstacle => presets::obstacle(size).into(),
                Preset::Inflow => presets::simple_inflow(size).into(),
            };
            Simulation::try_from(UnfinalizedSimulation {
                size,
                cell_size: [args.x_cell_width, args.y_cell_height],
                delt: args.delta_t,
                gamma: args.gamma,
                reynolds: args.reynolds,
                sor_absolute_epsilon: args.sor_epsilon,
                max_iterations: args.sor_max_iterations,
                initial_norm_squared: None,
                iterations: 0,
                time: 0.0,
                omega: args.omega,
                grid,
            })
            .unwrap()
        }
    }
}

pub async fn run(args: Args) {
    println!("Exécute des simulations...");

    let mut sim = get_sim(&args, Preset::Obstacle);

    println!("Grid size {} x {}", sim.size[0], sim.size[1]);

    let [w, h] = sim.size;

    let scaling = 4;

    let background_color = Color::from_hex(0xfdf6e3);

    let mut image = Image::gen_image_color(w as u16, h as u16, background_color);

    let texture = Texture2D::from_image(&image);

    let mut preset_index = 0;

    let mut ui_state = initialize_state();

    loop {
        let (mouse_x, mouse_y) = mouse_position();

        clear_background(background_color);

        root_ui().window(
            hash!(),
            Vec2::new(20., (h * scaling) as f32 + 105.),
            Vec2::new(200., 280.),
            |ui| {
                ui.group(hash!(), vec2(190.0, 275.0), |ui| {
                    ui.label(None, "Controls");

                    if ui.button(None, "Run / Pause") {
                        ui_state.keep_running = !ui_state.keep_running;
                    }
                    ui.group(hash!(), vec2(50.0, 50.0), |ui| {
                        if ui.button(None, "Slower") {
                            // There's a bad UI interaction that happens if this
                            // if is collapsed into the ui.button code above,
                            // I assume it has to do with short-circuiting the
                            // && somehow.
                            #[allow(clippy::collapsible_if)]
                            if ui_state.speed_multiplier > 1 {
                                ui_state.speed_multiplier -= 1;
                            }
                        }
                        if ui.button(None, "Faster") {
                            ui_state.speed_multiplier += 1;
                        }
                    });

                    if ui.button(None, "Run one simulation step") {
                        ui_state.run = true;
                    }

                    if ui.button(None, "Visualize Speed") {
                        ui_state.color_type = ColorType::Speed;
                    }
                    if ui.button(None, "Visualize Pressure") {
                        ui_state.color_type = ColorType::Pressure;
                    }
                    if ui.button(None, "Reset Simulation") {
                        ui_state.reset = true;
                    }
                    ui.combo_box(hash!(), "Preset", Preset::VARIANTS, &mut preset_index);
                    let desired_preset = Preset::try_from(preset_index).unwrap();
                    if ui_state.preset != desired_preset {
                        ui_state.reset = true;
                    }
                    ui_state.preset = desired_preset;
                    if ui.button(None, "Reset Simulation") {
                        ui_state.reset = true;
                    }
                    if ui.button(None, "Mouse Inspects") {
                        ui_state.mouse_state = MouseState::Inspection;
                    }
                    if ui.button(None, "Mouse Draws Boundaries") {
                        ui_state.mouse_state = MouseState::Boundary;
                    }
                    if ui.button(None, "Mouse Draws Fluid") {
                        ui_state.mouse_state = MouseState::Fluid;
                    }
                });
            },
        );

        if ui_state.reset {
            sim = get_sim(&args, ui_state.preset);
            ui_state.reset = false;
        }

        // Set to 1 in case the user asked to run one iteration.
        let mut speed_multiplier = 1;

        if ui_state.keep_running {
            ui_state.run = true;
            // Set to the multiplier in case the user asked to keep running the
            // simulation.
            speed_multiplier = ui_state.speed_multiplier;
        }

        if ui_state.run {
            for _ in 0..speed_multiplier {
                sim.run_simulation_tick().unwrap();
            }
            ui_state.run = false;
        }

        render_simulation(&sim, &mut image, w, h, ui_state.color_type);

        texture.update(&image);
        draw_texture_ex(
            &texture,
            0.,
            0.,
            background_color,
            DrawTextureParams {
                dest_size: Some(vec2((w * scaling) as f32, (h * scaling) as f32)),
                ..Default::default()
            },
        );

        let m_x = (mouse_x / (scaling as f32)) as usize;
        let m_y = (mouse_y / (scaling as f32)) as usize;

        if (m_x < w) && (m_y < h) {
            let inspect_cell_pressure = sim.grid.pressure[(m_x, m_y)];
            let inspect_cell_speed =
                (sim.grid.u[(m_x, m_y)].powi(2) + sim.grid.v[(m_x, m_y)].powi(2)).sqrt();
            draw_text(
                &format!(
                    "x: {:?}, y: {:?}, press: {:.2?}, speed: {:.2?}",
                    m_x, m_y, inspect_cell_pressure, inspect_cell_speed
                )
                .to_string(),
                20.0,
                (h * scaling) as f32 + 35.0,
                30.0,
                DARKGREEN,
            );

            if is_mouse_button_down(MouseButton::Left) {
                match ui_state.mouse_state {
                    MouseState::Boundary => draw_cells(
                        &mut sim.grid,
                        Cell::Boundary(BoundaryCell::NoSlip),
                        m_x,
                        m_y,
                    ),
                    MouseState::Fluid => draw_cells(&mut sim.grid, Cell::Fluid, m_x, m_y),
                    _ => {}
                }
            }
        }
        draw_text(
            &format!(
                "time: {:.2?}, iter: {:?}, speedup: {:?}",
                sim.time, sim.iterations, ui_state.speed_multiplier
            )
            .to_string(),
            20.0,
            (h * scaling) as f32 + 65.0,
            30.0,
            DARKGREEN,
        );

        next_frame().await
    }
}
