use crate::cell::Cell;
use crate::grid::SimulationGrid;
use crate::types::GridSize;
use ndarray::Array;

use std::iter;

/// Generate an empty simulation grid
pub fn zeroes(size: GridSize) -> SimulationGrid {
    // ndarray doesn't provide a way to construct an Array from a single
    // value other than zero. For the CellType Array, we have to do this
    // weird dance of creating a 1D Array from an iterator of size x * y,
    // then turning it into a 2D array with the dimensions x and y.
    let cell_type_grid_1d =
        Array::from_iter(iter::repeat_n(Cell::Fluid, size[0] * size[1]));
    let cell_type_grid = cell_type_grid_1d.into_shape_with_order(size).unwrap();

    SimulationGrid {
        size,
        pressure: Array::zeros(size),
        u: Array::zeros(size),
        v: Array::zeros(size),
        cell_type: cell_type_grid,
    }
}
