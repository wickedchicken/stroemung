use crate::cell::Cell;
use crate::grid::SimulationGrid;
use crate::types::GridSize;
use ndarray::Array;

/// Generate an empty simulation grid
pub fn empty(size: GridSize) -> SimulationGrid {
    SimulationGrid {
        size,
        pressure: Array::zeros(size),
        u: Array::zeros(size),
        v: Array::zeros(size),
        cell_type: Array::from_elem(size, Cell::Fluid),
    }
}
