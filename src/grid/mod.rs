use std::fmt;

use ndarray::Array;
use serde::Deserialize;
use serde::Serialize;

use crate::math::Real;
use crate::types::{GridArray, GridSize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationGrid {
    size: GridSize,
    pressure: GridArray<Real>,
}

impl SimulationGrid {
    pub fn new(size: GridSize) -> SimulationGrid {
        SimulationGrid {
            size,
            pressure: Array::zeros(size),
        }
    }
}

impl std::fmt::Display for SimulationGrid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Simulation grid {}x{}", self.size[0], self.size[1],)?;
        writeln!(f, "Pressure:{}", self.pressure)?;
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
    }

    #[test]
    fn deserialize() {
        use std::fs::File;
        use std::io::BufReader;
        use std::path::Path;

        let test_data_directory = Path::new(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("test_data");
        let test_filename = test_data_directory.join("simple_grid.json");
        let result: SimulationGrid =
            serde_json::from_reader(BufReader::new(File::open(test_filename).unwrap()))
                .unwrap();
        insta::assert_json_snapshot!(result);
    }

    #[test]
    fn serialize() {
        let size = [2, 3];
        let grid = SimulationGrid::new(size);
        insta::assert_json_snapshot!(grid);
    }
}
