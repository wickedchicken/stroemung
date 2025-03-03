pub mod presets;

use std::collections::BTreeSet;
use std::fmt;
use std::io::Read;

use serde::Deserialize;
use serde::Serialize;

use serde_json::Error as SerdeError;

use ndarray::Zip;
use thiserror::Error;

use crate::cell::Cell;
use crate::math::Real;
use crate::types::{BoundaryIndex, GridArray, GridIndex, GridSize};

type Neighbors = [Option<GridIndex>; 4];

#[derive(Error, Debug)]
pub enum SimulationGridError {
    #[error("An error occurred while deserializing: `{0}`")]
    DeserializationError(#[from] SerdeError),
    #[error("A cell `{0}` at `{1}` was not a BoundaryCell as expected.")]
    BoundaryListIncorrectError(String, String),
    #[error("A cell `{0}` at `{1}` has fluid on opposing sides.")]
    BoundaryTooThinError(String, String),
}

#[derive(Debug, Default)]
pub struct BoundaryList {
    boundaries: BTreeSet<BoundaryIndex>,
    pub sorted_boundary_list: Vec<(GridIndex, Neighbors)>,
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

// Useful for test code
impl From<SimulationGrid> for UnfinalizedSimulationGrid {
    fn from(item: SimulationGrid) -> Self {
        // Will be nicer once https://github.com/rust-lang/rust/issues/86555
        // is in stable.
        UnfinalizedSimulationGrid {
            size: item.size,
            pressure: item.pressure,
            u: item.u,
            v: item.v,
            cell_type: item.cell_type,
        }
    }
}

// This must be the same as UnfinalizedSimulationGrid, except for boundaries.
// We have two types to make sure we never deserialize without forgetting to
// generate the boundary list.
#[derive(Debug, Serialize)]
pub struct SimulationGrid {
    pub size: GridSize,
    pub pressure: GridArray<Real>,
    pub u: GridArray<Real>,
    pub v: GridArray<Real>,
    pub cell_type: GridArray<Cell>,
    #[serde(skip)]
    pub boundaries: BoundaryList,
}

impl TryFrom<UnfinalizedSimulationGrid> for SimulationGrid {
    type Error = SimulationGridError;

    fn try_from(item: UnfinalizedSimulationGrid) -> Result<Self, Self::Error> {
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
        grid.rebuild_boundary_list()?;
        Ok(grid)
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
    fn rebuild_boundary_list(&mut self) -> Result<(), SimulationGridError> {
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
        self.boundaries.sorted_boundary_list = self
            .boundaries
            .boundaries
            .iter()
            .copied()
            .map(|x| ((x.0, x.1), self.fluid_neighbors((x.0, x.1))))
            .collect();

        for (idx, [left, right, up, down]) in &self.boundaries.sorted_boundary_list {
            if (left.is_some() && right.is_some()) || (up.is_some() && down.is_some()) {
                return Err(SimulationGridError::BoundaryTooThinError(
                    self.cell_type[*idx].to_string(),
                    format!("{:?}", *idx),
                ));
            }
        }
        Ok(())
    }

    fn fluid_neighbors(&self, cell_idx: GridIndex) -> Neighbors {
        let left: Option<GridIndex> = if cell_idx.0 > 0 {
            let test_index = (cell_idx.0 - 1, cell_idx.1);
            match self.cell_type[test_index] {
                Cell::Fluid => Some(test_index),
                _ => None,
            }
        } else {
            None
        };

        let right: Option<GridIndex> = if cell_idx.0 < (self.size[0] - 1) {
            let test_index = (cell_idx.0 + 1, cell_idx.1);
            match self.cell_type[test_index] {
                Cell::Fluid => Some(test_index),
                _ => None,
            }
        } else {
            None
        };

        let up: Option<GridIndex> = if cell_idx.1 > 0 {
            let test_index = (cell_idx.0, cell_idx.1 - 1);
            match self.cell_type[test_index] {
                Cell::Fluid => Some(test_index),
                _ => None,
            }
        } else {
            None
        };

        let down: Option<GridIndex> = if cell_idx.1 < (self.size[1] - 1) {
            let test_index = (cell_idx.0, cell_idx.1 + 1);
            match self.cell_type[test_index] {
                Cell::Fluid => Some(test_index),
                _ => None,
            }
        } else {
            None
        };

        [left, right, up, down]
    }

    pub fn from_reader<R: Read>(
        reader: R,
    ) -> Result<SimulationGrid, SimulationGridError> {
        match serde_json::from_reader::<R, UnfinalizedSimulationGrid>(reader) {
            Ok(unfinalized) => SimulationGrid::try_from(unfinalized),
            Err(x) => Err(SimulationGridError::DeserializationError(x)),
        }
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
    fn thin_boundary() {
        use crate::cell::{BoundaryCell, Cell};
        let size = [3, 3];

        let boundaries: Vec<Vec<GridIndex>> =
            vec![vec![(1, 0), (1, 1), (1, 2)], vec![(0, 1), (1, 1), (2, 1)]];
        for example in &boundaries {
            let mut unfinalized = UnfinalizedSimulationGrid {
                size,
                pressure: Array::zeros(size),
                u: Array::zeros(size),
                v: Array::zeros(size),
                cell_type: Array::from_elem(size, Cell::Fluid),
            };
            for idx in example {
                unfinalized.cell_type[*idx] = Cell::Boundary(BoundaryCell::NoSlip);
            }
            let grid = SimulationGrid::try_from(unfinalized);
            assert!(grid.is_err());
            assert!(format!("{:?}", grid).contains("BoundaryTooThinError"));
        }
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

        let expected_neighbors: Vec<Neighbors> = vec![
            [None, None, None, None],
            [None, Some((1, 1)), None, None],
            [None, None, None, None],
            [None, None, None, Some((1, 1))],
            [None, None, Some((1, 1)), None],
            [None, None, None, None],
            [Some((1, 1)), None, None, None],
            [None, None, None, None],
        ];

        let expected_sorted_list: Vec<(GridIndex, Neighbors)> = expected_boundaries
            .iter()
            .zip(expected_neighbors)
            .map(|(x, y)| (*x, y))
            .collect();

        for idx in &expected_boundaries {
            unfinalized.cell_type[*idx] = Cell::Boundary(BoundaryCell::NoSlip);
        }

        let grid = SimulationGrid::try_from(unfinalized).unwrap();

        let calculated_boundaries_as_list: Vec<BoundaryIndex> =
            grid.boundaries.boundaries.iter().copied().collect();

        assert_eq!(calculated_boundaries_as_list, expected_boundary_indices);
        assert_eq!(grid.boundaries.sorted_boundary_list, expected_sorted_list);
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
