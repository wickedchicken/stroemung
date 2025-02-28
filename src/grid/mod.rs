pub mod presets;

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::io::Read;

use serde::Deserialize;
use serde::Serialize;

use ndarray::Zip;

use crate::cell::Cell;
use crate::math::Real;
use crate::types::{BoundaryIndex, GridArray, GridIndex, GridSize};

#[derive(Debug, Default)]
struct BoundaryList {
    boundaries: BTreeSet<BoundaryIndex>,
    sorted_boundary_list: Vec<GridIndex>,
}

impl std::fmt::Display for BoundaryList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Boundaries:")?;
        for elem in &self.boundaries {
            writeln!(f, "  {:?}", elem)?;
        }
        writeln!(f, "Sorted Boundary List:")?;
        for elem in &self.sorted_boundary_list {
            writeln!(f, "  {:?}", elem)?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnfinalizedSimulationGrid {
    size: GridSize,
    pressure: GridArray<Real>,
    u: GridArray<Real>,
    v: GridArray<Real>,
    cell_type: GridArray<Cell>,
}

// This must be the same as UnfinalizedSimulationGrid, except for boundaries.
// We have two types to make sure we never deserialize without forgetting to
// generate the boundary list.
#[derive(Debug, Serialize)]
pub struct SimulationGrid {
    size: GridSize,
    pressure: GridArray<Real>,
    u: GridArray<Real>,
    v: GridArray<Real>,
    cell_type: GridArray<Cell>,
    #[serde(skip)]
    boundaries: BoundaryList,
}

impl From<UnfinalizedSimulationGrid> for SimulationGrid {
    fn from(item: UnfinalizedSimulationGrid) -> Self {
        // Will be nicer once https://github.com/rust-lang/rust/issues/86555
        // is in stable.
        let mut grid = SimulationGrid {
            size: item.size,
            pressure: item.pressure,
            u: item.u,
            v: item.v,
            cell_type: item.cell_type,
            boundaries: Default::default(),
        };
        grid.rebuild_boundary_list();
        grid
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
    fn rebuild_boundary_list(&mut self) {
        self.boundaries.boundaries.clear();
        // Run a for_each with the value and indices. See
        // https://github.com/rust-ndarray/ndarray/issues/1093 for details.
        Zip::indexed(self.cell_type.view()).for_each(|idx, val| {
            if let Cell::Boundary(_) = val {
                self.boundaries
                    .boundaries
                    .insert(BoundaryIndex(idx.0, idx.1));
            }
        });
        dbg!(&self.boundaries.boundaries);
        self.boundaries.sorted_boundary_list = self
            .boundaries
            .boundaries
            .iter()
            .copied()
            .map(|x| (x.0, x.1))
            .collect();
    }

    pub fn from_reader<R: Read>(reader: R) -> Result<SimulationGrid, Box<dyn Error>> {
        let unfinalized: UnfinalizedSimulationGrid = serde_json::from_reader(reader)?;
        Ok(SimulationGrid::from(unfinalized))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::Array;
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
    fn rebuild_boundary_list() {
        use crate::cell::{BoundaryCell, Cell};
        let size = [3, 3];
        let mut unfinalized = UnfinalizedSimulationGrid {
            size,
            pressure: Array::zeros(size),
            u: Array::zeros(size),
            v: Array::zeros(size),
            cell_type: Array::from_elem(size, Cell::Fluid),
        };

        // Everything except for the middle cell
        let expected_boundaries: Vec<GridIndex> = vec![
            (0, 0),
            (0, 1),
            (0, 2),
            (1, 0),
            (1, 2),
            (2, 0),
            (2, 1),
            (2, 2),
        ];
        let expected_boundary_indices: Vec<BoundaryIndex> = expected_boundaries
            .iter()
            .map(|x| BoundaryIndex(x.0, x.1))
            .collect();

        for idx in &expected_boundaries {
            unfinalized.cell_type[*idx] = Cell::Boundary(BoundaryCell::NoSlip);
        }

        let grid = SimulationGrid::from(unfinalized);

        let calculated_boundaries_as_list: Vec<BoundaryIndex> =
            grid.boundaries.boundaries.iter().copied().collect();

        assert_eq!(calculated_boundaries_as_list, expected_boundary_indices);
        assert_eq!(grid.boundaries.sorted_boundary_list, expected_boundaries);
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
    fn deserialize_boundaries() {
        let test_filename = test_data_directory().join("small_grid_with_boundaries.json");
        let result = SimulationGrid::from_reader(BufReader::new(
            File::open(test_filename).unwrap(),
        ))
        .unwrap();

        insta::assert_json_snapshot!(result);
        insta::assert_snapshot!(result.boundaries);
    }

    #[test]
    fn serialize() {
        let size = [2, 3];
        let grid = presets::empty(size);
        insta::assert_json_snapshot!(grid);
    }
}
