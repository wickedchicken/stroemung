use crate::grid::SimulationGrid;
use crate::types::GridSize;
use ndarray::Array;

/// Generate an empty simulation grid
pub fn zeroes(size: GridSize) -> SimulationGrid {
    SimulationGrid {
        size,
        pressure: Array::zeros(size),
    }
}
