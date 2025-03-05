use crate::cell::{BoundaryCell, Cell};
use crate::grid::{SimulationGrid, UnfinalizedSimulationGrid};
use crate::math::Real;
use crate::types::GridSize;
use ndarray::{Array, Ix2};

/// Generate an empty simulation grid
pub fn empty(size: GridSize) -> SimulationGrid {
    SimulationGrid::try_from(UnfinalizedSimulationGrid {
        size,
        pressure: Array::zeros(size),
        u: Array::zeros(size),
        v: Array::zeros(size),
        cell_type: Array::from_elem(size, Cell::Fluid),
    })
    .unwrap()
}

pub fn simple_inflow(size: GridSize) -> SimulationGrid {
    let mut cell_array = Array::from_elem(size, Cell::Fluid);
    for x in 0..size[0] {
        cell_array[(x, 0)] = Cell::Boundary(BoundaryCell::NoSlip);
        cell_array[(x, size[1] - 1)] = Cell::Boundary(BoundaryCell::NoSlip);
    }
    for y in 1..(size[1] - 1) {
        cell_array[(0, y)] = Cell::Boundary(BoundaryCell::Inflow {
            velocity: [1.0, 0.0],
        });
        cell_array[(size[0] - 1, y)] = Cell::Boundary(BoundaryCell::Outflow);
    }

    SimulationGrid::try_from(UnfinalizedSimulationGrid {
        size,
        pressure: Array::zeros(size),
        u: Array::zeros(size),
        v: Array::zeros(size),
        cell_type: cell_array,
    })
    .unwrap()
}

fn draw_circle(cell_array: &mut Array<Cell, Ix2>, x: usize, y: usize, radius: Real) {
    let (x_size, y_size) = cell_array.dim();
    for xi in (x.saturating_sub(radius as usize))..(x.saturating_add(radius as usize)) {
        if xi >= x_size {
            continue;
        }
        let x_dist = xi as i32 - x as i32;
        for yi in (y.saturating_sub(radius as usize))..(y.saturating_add(radius as usize))
        {
            if yi >= y_size {
                continue;
            }
            let y_dist = yi as i32 - y as i32;
            let distance = ((x_dist * x_dist + y_dist * y_dist) as f64).sqrt();

            if distance < radius {
                cell_array[(xi, yi)] = Cell::Boundary(BoundaryCell::NoSlip);
            }
        }
    }
}

pub fn obstacle(size: GridSize) -> SimulationGrid {
    let mut cell_array = Array::from_elem(size, Cell::Fluid);
    for x in 0..size[0] {
        cell_array[(x, 0)] = Cell::Boundary(BoundaryCell::NoSlip);
        cell_array[(x, size[1] - 1)] = Cell::Boundary(BoundaryCell::NoSlip);
    }
    for y in 1..(size[1] - 1) {
        cell_array[(0, y)] = Cell::Boundary(BoundaryCell::Inflow {
            velocity: [1.0, 0.0],
        });
        cell_array[(size[0] - 1, y)] = Cell::Boundary(BoundaryCell::Outflow);
    }

    draw_circle(&mut cell_array, 20, size[1] / 2, 5.0);

    SimulationGrid::try_from(UnfinalizedSimulationGrid {
        size,
        pressure: Array::zeros(size),
        u: Array::zeros(size),
        v: Array::zeros(size),
        cell_type: cell_array,
    })
    .unwrap()
}
