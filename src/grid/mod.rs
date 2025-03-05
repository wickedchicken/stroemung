pub mod presets;

use std::collections::BTreeSet;
use std::fmt;
use std::io::Read;

use serde::Deserialize;
use serde::Serialize;

use serde_json::Error as SerdeError;

use ndarray::Zip;
use thiserror::Error;

use crate::cell::{BoundaryCell, Cell};
use crate::math::Real;
use crate::types::{BoundaryIndex, GridArray, GridIndex, GridSize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
    North {
        north_neighbor: GridIndex,
    },
    NorthEast {
        north_neighbor: GridIndex,
        east_neighbor: GridIndex,
    },
    East {
        east_neighbor: GridIndex,
    },
    SouthEast {
        south_neighbor: GridIndex,
        east_neighbor: GridIndex,
    },
    South {
        south_neighbor: GridIndex,
    },
    SouthWest {
        south_neighbor: GridIndex,
        west_neighbor: GridIndex,
    },
    West {
        west_neighbor: GridIndex,
    },
    NorthWest {
        north_neighbor: GridIndex,
        west_neighbor: GridIndex,
    },
}

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
    pub sorted_boundary_list: Vec<(GridIndex, Option<EdgeType>)>,
    pub fluid_cells: Real,
    // This is scratch space so the vector doesn't keep getting reallocated
    // between simulation steps
    pub u_v_restore: Vec<(GridIndex, Option<Real>, Option<Real>)>,
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
            boundaries: BoundaryList {
                boundaries: Default::default(),
                sorted_boundary_list: Default::default(),
                u_v_restore: Vec::new(),
                fluid_cells: 0.0,
            },
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
    fn neighbors(&self, idx: GridIndex) -> [Option<(GridIndex, Cell)>; 4] {
        // Note that we use the convention that 0,0 is the upper-left corner
        // instead of the bottom left as in the book. This means that "north"
        // here refers to j-1, while that is is "south" in the book.
        let north: Option<(GridIndex, Cell)> = if idx.1 > 0 {
            let test_index = (idx.0, idx.1 - 1);
            Some((test_index, self.cell_type[test_index]))
        } else {
            None
        };

        let south: Option<(GridIndex, Cell)> = if idx.1 < (self.size[1] - 1) {
            let test_index = (idx.0, idx.1 + 1);
            Some((test_index, self.cell_type[test_index]))
        } else {
            None
        };

        let east: Option<(GridIndex, Cell)> = if idx.0 < (self.size[0] - 1) {
            let test_index = (idx.0 + 1, idx.1);
            Some((test_index, self.cell_type[test_index]))
        } else {
            None
        };

        let west: Option<(GridIndex, Cell)> = if idx.0 > 0 {
            let test_index = (idx.0 - 1, idx.1);
            Some((test_index, self.cell_type[test_index]))
        } else {
            None
        };

        [north, south, east, west]
    }

    fn rebuild_boundary_list(&mut self) -> Result<(), SimulationGridError> {
        let mut fluid_cells = 0;
        self.boundaries.boundaries.clear();
        self.boundaries.u_v_restore = Vec::new();
        // Run a for_each with the value and indices. See
        // https://github.com/rust-ndarray/ndarray/issues/1093 for details.
        Zip::indexed(self.cell_type.view()).for_each(|idx, val| {
            if let Cell::Boundary(_) = val {
                self.boundaries
                    .boundaries
                    .insert(BoundaryIndex(idx.0, idx.1));
            } else {
                fluid_cells += 1;
            }
        });

        let get_neighbors = |idx: BoundaryIndex| {
            let new_idx: GridIndex = (idx.0, idx.1);
            let edge_type = self.calculate_edges(new_idx)?;
            Ok::<((usize, usize), std::option::Option<EdgeType>), SimulationGridError>((
                new_idx, edge_type,
            ))
        };
        let result: Result<Vec<_>, _> = self
            .boundaries
            .boundaries
            .iter()
            .copied()
            .map(get_neighbors)
            .collect();
        self.boundaries.sorted_boundary_list = result?;
        self.boundaries.fluid_cells = fluid_cells as Real;
        Ok(())
    }

    fn calculate_edges(
        &self,
        cell_idx: GridIndex,
    ) -> Result<Option<EdgeType>, SimulationGridError> {
        let [north_neighbor, south_neighbor, east_neighbor, west_neighbor] =
            self.neighbors(cell_idx);

        let left: Option<GridIndex> = match west_neighbor {
            Some((idx, Cell::Fluid)) => Some(idx),
            _ => None,
        };

        let right: Option<GridIndex> = match east_neighbor {
            Some((idx, Cell::Fluid)) => Some(idx),
            _ => None,
        };

        let up: Option<GridIndex> = match north_neighbor {
            Some((idx, Cell::Fluid)) => Some(idx),
            _ => None,
        };

        let down: Option<GridIndex> = match south_neighbor {
            Some((idx, Cell::Fluid)) => Some(idx),
            _ => None,
        };

        match (left, right, up, down) {
            (None, None, None, None) => Ok(None),
            (Some(left), None, None, None) => Ok(Some(EdgeType::West {
                west_neighbor: left,
            })),
            (Some(left), None, Some(up), None) => Ok(Some(EdgeType::NorthWest {
                north_neighbor: up,
                west_neighbor: left,
            })),
            (None, None, Some(up), None) => {
                Ok(Some(EdgeType::North { north_neighbor: up }))
            }
            (None, Some(right), Some(up), None) => Ok(Some(EdgeType::NorthEast {
                north_neighbor: up,
                east_neighbor: right,
            })),
            (None, Some(right), None, None) => Ok(Some(EdgeType::East {
                east_neighbor: right,
            })),
            (None, Some(right), None, Some(down)) => Ok(Some(EdgeType::SouthEast {
                south_neighbor: down,
                east_neighbor: right,
            })),
            (None, None, None, Some(down)) => Ok(Some(EdgeType::South {
                south_neighbor: down,
            })),
            (Some(left), None, None, Some(down)) => Ok(Some(EdgeType::SouthWest {
                south_neighbor: down,
                west_neighbor: left,
            })),
            _ => Err(SimulationGridError::BoundaryTooThinError(
                self.cell_type[cell_idx].to_string(),
                format!("{:?}", cell_idx),
            )),
        }
    }

    pub fn from_reader<R: Read>(
        reader: R,
    ) -> Result<SimulationGrid, SimulationGridError> {
        match serde_json::from_reader::<R, UnfinalizedSimulationGrid>(reader) {
            Ok(unfinalized) => SimulationGrid::try_from(unfinalized),
            Err(x) => Err(SimulationGridError::DeserializationError(x)),
        }
    }

    pub fn copy_pressure_to_boundaries(&mut self) -> Result<(), SimulationGridError> {
        for (boundary_idx, maybe_edge) in &self.boundaries.sorted_boundary_list {
            // Don't do anything if we're not on a boundary.
            let Some(edge) = maybe_edge else {
                continue;
            };
            match self.cell_type[*boundary_idx] {
                Cell::Boundary(_) => {
                    match edge {
                        EdgeType::North { north_neighbor } => {
                            self.pressure[*boundary_idx] = self.pressure[*north_neighbor]
                        }
                        EdgeType::NorthEast {
                            north_neighbor,
                            east_neighbor,
                        } => {
                            self.pressure[*boundary_idx] = (self.pressure
                                [*north_neighbor]
                                + self.pressure[*east_neighbor])
                                / 2.0
                        }
                        EdgeType::East { east_neighbor } => {
                            self.pressure[*boundary_idx] = self.pressure[*east_neighbor]
                        }
                        EdgeType::SouthEast {
                            south_neighbor,
                            east_neighbor,
                        } => {
                            self.pressure[*boundary_idx] = (self.pressure
                                [*south_neighbor]
                                + self.pressure[*east_neighbor])
                                / 2.0
                        }
                        EdgeType::South { south_neighbor } => {
                            self.pressure[*boundary_idx] = self.pressure[*south_neighbor]
                        }
                        EdgeType::SouthWest {
                            south_neighbor,
                            west_neighbor,
                        } => {
                            self.pressure[*boundary_idx] = (self.pressure
                                [*south_neighbor]
                                + self.pressure[*west_neighbor])
                                / 2.0
                        }
                        EdgeType::West { west_neighbor } => {
                            self.pressure[*boundary_idx] = self.pressure[*west_neighbor]
                        }
                        EdgeType::NorthWest {
                            north_neighbor,
                            west_neighbor,
                        } => {
                            self.pressure[*boundary_idx] = (self.pressure
                                [*north_neighbor]
                                + self.pressure[*west_neighbor])
                                / 2.0
                        }
                    };
                }
                other => {
                    return Err(SimulationGridError::BoundaryListIncorrectError(
                        other.to_string(),
                        format!("{:?}", *boundary_idx),
                    ))
                }
            };
        }

        Ok(())
    }

    pub fn set_boundary_u_and_v(&mut self) -> Result<(), SimulationGridError> {
        // We're going to copy u and v back into the vector in the loop
        self.boundaries.u_v_restore.clear();

        for (boundary_idx, maybe_edge) in &self.boundaries.sorted_boundary_list {
            // Don't do anything if we're not on a boundary.
            let Some(edge) = maybe_edge else {
                // We still want to restore its u and v though.
                self.boundaries.u_v_restore.push((
                    *boundary_idx,
                    Some(self.u[*boundary_idx]),
                    Some(self.v[*boundary_idx]),
                ));
                continue;
            };
            // There are n+1 edges for n cells in a row. To prevent
            // off-by-one errors in updating the edges of the boundary,
            // we designate the "north" and "west" edges the starting
            // points. This means that *corners* with a north or west
            // edge are responsible for updating an extra v or u edge
            // respectively. A NorthWest cell must update both extra
            // u and v edges.
            match self.cell_type[*boundary_idx] {
                Cell::Boundary(BoundaryCell::NoSlip) => {
                    let boundary_u = 0.0;
                    let boundary_v = 0.0;

                    match edge {
                        EdgeType::North { north_neighbor } => {
                            self.u[*boundary_idx] = -self.u[*north_neighbor];
                            self.v[*north_neighbor] = boundary_v;
                        }
                        EdgeType::NorthEast {
                            north_neighbor,
                            east_neighbor,
                        } => {
                            self.u[*boundary_idx] = boundary_u;
                            self.v[*north_neighbor] = boundary_v;
                            self.v[*boundary_idx] = -self.v[*east_neighbor];
                        }
                        EdgeType::East { east_neighbor } => {
                            self.u[*boundary_idx] = boundary_u;
                            self.v[*boundary_idx] = -self.v[*east_neighbor];
                        }
                        EdgeType::SouthEast { .. } => {
                            self.u[*boundary_idx] = boundary_u;
                            self.v[*boundary_idx] = boundary_v;
                        }
                        EdgeType::South { south_neighbor } => {
                            self.u[*boundary_idx] = -self.u[*south_neighbor];
                            self.v[*boundary_idx] = boundary_v;
                        }
                        EdgeType::SouthWest {
                            south_neighbor,
                            west_neighbor,
                        } => {
                            self.u[*west_neighbor] = boundary_u;
                            self.u[*boundary_idx] = -self.u[*south_neighbor];
                            self.v[*boundary_idx] = boundary_v;
                        }
                        EdgeType::West { west_neighbor } => {
                            self.u[*west_neighbor] = boundary_u;
                            self.v[*boundary_idx] = -self.v[*west_neighbor];
                        }
                        EdgeType::NorthWest {
                            north_neighbor,
                            west_neighbor,
                        } => {
                            self.u[*west_neighbor] = boundary_u;
                            self.u[*boundary_idx] = -self.u[*north_neighbor];
                            self.v[*north_neighbor] = boundary_v;
                            self.v[*boundary_idx] = -self.v[*west_neighbor];
                        }
                    };
                }
                Cell::Boundary(BoundaryCell::Outflow) => {
                    match edge {
                        EdgeType::North { north_neighbor } => {
                            self.u[*boundary_idx] = self.u[*north_neighbor];
                            self.v[*boundary_idx] = self.v[*north_neighbor];
                        }
                        EdgeType::NorthEast {
                            north_neighbor,
                            east_neighbor,
                        } => {
                            self.u[*boundary_idx] = self.u[*north_neighbor];
                            self.v[*boundary_idx] = self.v[*east_neighbor];
                        }
                        EdgeType::East { east_neighbor } => {
                            self.u[*boundary_idx] = self.u[*east_neighbor];
                            self.v[*boundary_idx] = self.v[*east_neighbor];
                        }
                        EdgeType::SouthEast {
                            south_neighbor,
                            east_neighbor,
                        } => {
                            self.u[*boundary_idx] = self.u[*east_neighbor];
                            self.v[*boundary_idx] = self.v[*south_neighbor];
                        }
                        EdgeType::South { south_neighbor } => {
                            self.u[*boundary_idx] = self.u[*south_neighbor];
                            self.v[*boundary_idx] = self.v[*south_neighbor];
                        }
                        EdgeType::SouthWest {
                            south_neighbor,
                            west_neighbor,
                        } => {
                            self.u[*boundary_idx] = self.u[*west_neighbor];
                            self.v[*boundary_idx] = self.v[*south_neighbor];
                        }
                        EdgeType::West { west_neighbor } => {
                            self.u[*boundary_idx] = self.u[*west_neighbor];
                            self.v[*boundary_idx] = self.v[*west_neighbor];
                        }
                        EdgeType::NorthWest {
                            north_neighbor,
                            west_neighbor,
                        } => {
                            self.u[*boundary_idx] = self.u[*north_neighbor];
                            self.v[*boundary_idx] = self.v[*west_neighbor];
                        }
                    };
                }
                Cell::Boundary(BoundaryCell::Inflow { velocity }) => {
                    let [boundary_u, boundary_v] = velocity;
                    match edge {
                        EdgeType::North { north_neighbor } => {
                            self.u[*boundary_idx] = -self.u[*north_neighbor];
                            self.v[*north_neighbor] = boundary_v;
                        }
                        EdgeType::NorthEast {
                            north_neighbor,
                            east_neighbor,
                        } => {
                            self.u[*boundary_idx] = boundary_u;
                            self.v[*north_neighbor] = boundary_v;
                            self.v[*boundary_idx] = -self.v[*east_neighbor];
                        }
                        EdgeType::East { east_neighbor } => {
                            self.u[*boundary_idx] = boundary_u;
                            self.v[*boundary_idx] = -self.v[*east_neighbor];
                        }
                        EdgeType::SouthEast { .. } => {
                            self.u[*boundary_idx] = boundary_u;
                            self.v[*boundary_idx] = boundary_v;
                        }
                        EdgeType::South { south_neighbor } => {
                            self.u[*boundary_idx] = -self.u[*south_neighbor];
                            self.v[*boundary_idx] = boundary_v;
                        }
                        EdgeType::SouthWest {
                            south_neighbor,
                            west_neighbor,
                        } => {
                            self.u[*west_neighbor] = boundary_u;
                            self.u[*boundary_idx] = -self.u[*south_neighbor];
                            self.v[*boundary_idx] = boundary_v;
                        }
                        EdgeType::West { west_neighbor } => {
                            self.u[*west_neighbor] = boundary_u;
                            self.v[*boundary_idx] = -self.v[*west_neighbor];
                        }
                        EdgeType::NorthWest {
                            north_neighbor,
                            west_neighbor,
                        } => {
                            self.u[*west_neighbor] = boundary_u;
                            self.u[*boundary_idx] = -self.u[*north_neighbor];
                            self.v[*north_neighbor] = boundary_v;
                            self.v[*boundary_idx] = -self.v[*west_neighbor];
                        }
                    };
                }
                other => {
                    return Err(SimulationGridError::BoundaryListIncorrectError(
                        other.to_string(),
                        format!("{:?}", *boundary_idx),
                    ))
                }
            }

            self.boundaries.u_v_restore.push((
                *boundary_idx,
                Some(self.u[*boundary_idx]),
                Some(self.v[*boundary_idx]),
            ));

            // Stash u and v values for later restoration
            match edge {
                EdgeType::North { north_neighbor } => {
                    self.boundaries.u_v_restore.push((
                        *boundary_idx,
                        None,
                        Some(self.v[*north_neighbor]),
                    ));
                }
                EdgeType::NorthEast {
                    north_neighbor,
                    east_neighbor: _,
                } => {
                    self.boundaries.u_v_restore.push((
                        *boundary_idx,
                        None,
                        Some(self.v[*north_neighbor]),
                    ));
                }
                EdgeType::SouthWest {
                    south_neighbor: _,
                    west_neighbor,
                } => {
                    self.boundaries.u_v_restore.push((
                        *boundary_idx,
                        Some(self.u[*west_neighbor]),
                        None,
                    ));
                }
                EdgeType::West { west_neighbor } => {
                    self.boundaries.u_v_restore.push((
                        *boundary_idx,
                        Some(self.u[*west_neighbor]),
                        None,
                    ));
                }
                EdgeType::NorthWest {
                    north_neighbor,
                    west_neighbor,
                } => {
                    self.boundaries.u_v_restore.push((
                        *boundary_idx,
                        Some(self.u[*west_neighbor]),
                        Some(self.v[*north_neighbor]),
                    ));
                }
                _ => {}
            };
        }
        Ok(())
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

        let examples = vec![
            (
                // Everything except for the middle cell is a boundary
                vec![
                    (0, 0),
                    (0, 1),
                    (0, 2),
                    (1, 0),
                    (1, 2),
                    (2, 0),
                    (2, 1),
                    (2, 2),
                ],
                vec![
                    None,
                    Some(EdgeType::East {
                        east_neighbor: (1, 1),
                    }),
                    None,
                    Some(EdgeType::South {
                        south_neighbor: (1, 1),
                    }),
                    Some(EdgeType::North {
                        north_neighbor: (1, 1),
                    }),
                    None,
                    Some(EdgeType::West {
                        west_neighbor: (1, 1),
                    }),
                    None,
                ],
            ),
            (
                // All corners are boundaries
                vec![(0, 0), (0, 2), (2, 0), (2, 2)],
                vec![
                    Some(EdgeType::SouthEast {
                        south_neighbor: (0, 1),
                        east_neighbor: (1, 0),
                    }),
                    Some(EdgeType::NorthEast {
                        north_neighbor: (0, 1),
                        east_neighbor: (1, 2),
                    }),
                    Some(EdgeType::SouthWest {
                        south_neighbor: (2, 1),
                        west_neighbor: (1, 0),
                    }),
                    Some(EdgeType::NorthWest {
                        north_neighbor: (2, 1),
                        west_neighbor: (1, 2),
                    }),
                ],
            ),
        ];

        for (expected_boundaries, expected_neighbors) in examples {
            let mut unfinalized = UnfinalizedSimulationGrid {
                size,
                pressure: Array::zeros(size),
                u: Array::zeros(size),
                v: Array::zeros(size),
                cell_type: Array::from_elem(size, Cell::Fluid),
            };

            let expected_boundary_indices: Vec<BoundaryIndex> = expected_boundaries
                .iter()
                .map(|x| BoundaryIndex(x.0, x.1))
                .collect();

            let expected_sorted_list: Vec<(GridIndex, Option<EdgeType>)> =
                expected_boundaries
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
