use std::fmt;
use std::iter;

use ndarray::Array;
use serde::Deserialize;
use serde::Serialize;

use crate::cell::Cell;
use crate::math::Real;
use crate::types::{GridArray, GridSize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationGrid {
    size: GridSize,
    pressure: GridArray<Real>,
    u: GridArray<Real>,
    v: GridArray<Real>,
    cell_type: GridArray<Cell>,
}

impl SimulationGrid {
    pub fn new(size: GridSize) -> SimulationGrid {
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
}

impl std::fmt::Display for SimulationGrid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Simulation grid {}x{}", self.size[0], self.size[1],)?;
        writeln!(f, "Pressure:{}", self.pressure)?;
        writeln!(f, "u:{}", self.u)?;
        writeln!(f, "v:{}", self.v)?;
        writeln!(f, "Cell Type:{}", self.cell_type)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_size() {
        let size = [5, 7];
        let grid = SimulationGrid::new(size);
        assert_eq!(grid.size[0], 5);
        assert_eq!(grid.size[1], 7);
        assert_eq!(grid.pressure.shape(), size);
        assert_eq!(grid.u.shape(), size);
        assert_eq!(grid.v.shape(), size);
        assert_eq!(grid.cell_type.shape(), size);
    }
}
