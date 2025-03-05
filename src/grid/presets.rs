use crate::cell::{BoundaryCell, Cell};
use crate::grid::{SimulationGrid, UnfinalizedSimulationGrid};
use crate::types::GridSize;
use ndarray::Array;

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
