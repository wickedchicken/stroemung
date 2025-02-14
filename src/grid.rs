use std::fmt;

use crate::math::Real;
use ndarray::{Array, Ix2};

#[derive(Debug)]
pub struct SimulationGrid {
    x_cells: usize,
    y_cells: usize,
    pressure: Array<Real, Ix2>,
}

impl SimulationGrid {
    pub fn new(x_cells: usize, y_cells: usize) -> SimulationGrid {
        SimulationGrid {
            x_cells,
            y_cells,
            pressure: Array::zeros((x_cells, y_cells)),
        }
    }
}

impl std::fmt::Display for SimulationGrid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Simulation grid {}x{}\nPressure:\n{}",
            self.x_cells, self.y_cells, self.pressure
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_size() {
        let grid = SimulationGrid::new(5, 7);
        assert_eq!(grid.x_cells, 5);
        assert_eq!(grid.y_cells, 7);
        assert_eq!(grid.pressure.shape(), [5, 7]);
    }
}
