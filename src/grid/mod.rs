pub mod presets;

use std::error::Error;
use std::fmt;
use std::io::Read;

use serde::Deserialize;
use serde::Serialize;

use crate::cell::Cell;
use crate::math::Real;
use crate::types::{GridArray, GridSize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UnfinalizedSimulationGrid {
    size: GridSize,
    pressure: GridArray<Real>,
    u: GridArray<Real>,
    v: GridArray<Real>,
    cell_type: GridArray<Cell>,
}

// A wrapper struct to make sure that we never deserialize without forgetting
// to do some postprocessing.
#[derive(Debug, Serialize)]
pub struct SimulationGrid {
    size: GridSize,
    pressure: GridArray<Real>,
    u: GridArray<Real>,
    v: GridArray<Real>,
    cell_type: GridArray<Cell>,
}

impl From<UnfinalizedSimulationGrid> for SimulationGrid {
    fn from(item: UnfinalizedSimulationGrid) -> Self {
        // Will be nicer once https://github.com/rust-lang/rust/issues/86555
        // is in stable.
        SimulationGrid {
            size: item.size,
            pressure: item.pressure,
            u: item.u,
            v: item.v,
            cell_type: item.cell_type,
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

impl SimulationGrid {
    pub fn from_reader<R: Read>(reader: R) -> Result<SimulationGrid, Box<dyn Error>> {
        let unfinalized: UnfinalizedSimulationGrid = serde_json::from_reader(reader)?;
        Ok(SimulationGrid::from(unfinalized))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::{Path, PathBuf};

    fn test_data_directory() -> PathBuf {
        Path::new(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("test_data")
    }

    #[test]
    fn grid_size() {
        let size = [5, 7];
        let grid = presets::empty(size);
        assert_eq!(grid.size[0], 5);
        assert_eq!(grid.size[1], 7);
        assert_eq!(grid.pressure.shape(), size);
        assert_eq!(grid.u.shape(), size);
        assert_eq!(grid.v.shape(), size);
        assert_eq!(grid.cell_type.shape(), size);
    }

    #[test]
    fn deserialize() {
        let test_filename = test_data_directory().join("simple_grid.json");
        let result = SimulationGrid::from_reader(BufReader::new(
            File::open(test_filename).unwrap(),
        ))
        .unwrap();
        insta::assert_json_snapshot!(result);
    }

    #[test]
    fn serialize() {
        let size = [2, 3];
        let grid = presets::empty(size);
        insta::assert_json_snapshot!(grid);
    }
}
