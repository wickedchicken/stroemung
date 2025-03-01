use std::error::Error;
use std::fmt;
use std::io::Read;

use crate::math::Real;
use crate::math::{du2dx, duvdx, duvdy, dv2dy, laplacian};

use serde::Deserialize;
use serde::Serialize;

use crate::grid::{SimulationGrid, UnfinalizedSimulationGrid};
use crate::types::{CellPhysicalSize, GridSize};

use ndarray::ArrayView2;

#[derive(Debug, Deserialize)]
pub struct UnfinalizedSimulation {
    size: GridSize,
    cell_size: CellPhysicalSize,
    delt: Real,
    gamma: Real,
    reynolds: Real,
    grid: UnfinalizedSimulationGrid,
}

// This must be the same as UnfinalizedSimulation, except the type
// of grid and without the calculated values. We have two types to make sure
// we never deserialize without forgetting to generate the boundary list.
#[derive(Debug, Serialize)]
pub struct Simulation {
    size: GridSize,
    cell_size: CellPhysicalSize,
    delt: Real,
    gamma: Real,
    reynolds: Real,
    grid: SimulationGrid,
}

impl From<UnfinalizedSimulation> for Simulation {
    fn from(item: UnfinalizedSimulation) -> Self {
        // Will be nicer once https://github.com/rust-lang/rust/issues/86555
        // is in stable.
        Simulation {
            size: item.size,
            cell_size: item.cell_size,
            delt: item.delt,
            gamma: item.gamma,
            reynolds: item.reynolds,
            grid: item.grid.into(),
        }
    }
}

impl std::fmt::Display for Simulation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "Simulation {}x{} cells, {}x{} physical size",
            self.size[0], self.size[1], self.cell_size[0], self.cell_size[1]
        )?;
        writeln!(f, "Time step delta:{}", self.delt)?;
        writeln!(f, "Gamma:{}", self.gamma)?;
        writeln!(f, "Reynolds number:{}", self.reynolds)?;
        writeln!(f, "{}", self.grid)?;
        Ok(())
    }
}

impl Simulation {
    pub fn from_reader<R: Read>(reader: R) -> Result<Simulation, Box<dyn Error>> {
        let unfinalized: UnfinalizedSimulation = serde_json::from_reader(reader)?;
        Ok(Simulation::from(unfinalized))
    }
}

/// Calculate F (the horizontal non-pressure part of the momentum equation)
///
/// # Arguments
///
/// * `u_view` - A 3x3-element ArrayView2 representing
///   u[(i-1) to (i+1), (j-1) to (j+1)].
/// * `v_view` - A 3x3-element ArrayView2 representing
///   v[(i-1) to (i+1), (j-1) to (j+1)].
/// * `delx` - "delta x," the physical width of the cell
/// * `dely` - "delta y," the physical width of the cell
/// * `delt` - "delta t," the amount of time per time step
/// * `gamma` - Greek letter gamma, the upwind discretization parameter
/// * `reynolds` - The Reynolds number for the simulation
pub fn calculate_f(
    u_view: ArrayView2<Real>,
    v_view: ArrayView2<Real>,
    delx: Real,
    dely: Real,
    delt: Real,
    gamma: Real,
    reynolds: Real,
) -> Real {
    u_view[(1, 1)]
        + (delt
            * ((laplacian(u_view, delx, dely) / reynolds)
                - du2dx(u_view, delx, gamma)
                - duvdy(u_view, v_view, dely, gamma)))
}

/// Calculate G (the vertical non-pressure part of the momentum equation)
///
/// # Arguments
///
/// * `u_view` - A 3x3-element ArrayView2 representing
///   u[(i-1) to (i+1), (j-1) to (j+1)].
/// * `v_view` - A 3x3-element ArrayView2 representing
///   v[(i-1) to (i+1), (j-1) to (j+1)].
/// * `delx` - "delta x," the physical width of the cell
/// * `dely` - "delta y," the physical width of the cell
/// * `delt` - "delta t," the amount of time per time step
/// * `gamma` - Greek letter gamma, the upwind discretization parameter
/// * `reynolds` - The Reynolds number for the simulation
pub fn calculate_g(
    u_view: ArrayView2<Real>,
    v_view: ArrayView2<Real>,
    delx: Real,
    dely: Real,
    delt: Real,
    gamma: Real,
    reynolds: Real,
) -> Real {
    v_view[(1, 1)]
        + (delt
            * ((laplacian(v_view, delx, dely) / reynolds)
                - duvdx(u_view, v_view, delx, gamma)
                - dv2dy(v_view, dely, gamma)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::{array, ArrayView2};
    use std::fs::File;
    use std::io::BufReader;
    use std::path::{Path, PathBuf};

    use crate::grid::presets;

    fn test_data_directory() -> PathBuf {
        Path::new(file!()).parent().unwrap().join("test_data")
    }

    #[test]
    fn deserialize() {
        let test_filename = test_data_directory().join("simple_simulation.json");
        let result =
            Simulation::from_reader(BufReader::new(File::open(test_filename).unwrap()))
                .unwrap();
        insta::assert_json_snapshot!(result);
    }

    #[test]
    fn serialize() {
        let size = [5, 7];
        let cell_size = [1., 2.];
        let delt = 1.4;
        let gamma = 1.7;
        let reynolds = 100.;

        let simulation = Simulation::from(UnfinalizedSimulation {
            size,
            cell_size,
            delt,
            gamma,
            reynolds,
            grid: presets::empty(size).into(),
        });

        insta::assert_json_snapshot!(simulation);
    }

    #[test]
    fn test_calculate_f() {
        // These don't have any particular significance, just some random data.
        let test_cases = [
            (
                array![[1., 2., 3.], [4., 5., 6.], [7., 8., 9.]],
                array![[8., 9., 10.], [11., 12., 13.], [14., 15., 16.]],
                1.,
                1.,
                0.005,
                1.7,
                100.,
                4.802500,
            ),
            (
                array![[1., 2., 3.], [4., 5., -6.], [-7., 8., 9.]],
                array![[8., 9., 10.], [11., -12., 13.], [14., 15., -16.]],
                1.,
                1.,
                0.006,
                1.7,
                10.,
                5.052800,
            ),
            (
                array![[1., 2., 3.], [4., 5., 6.], [7., 8., 9.]],
                array![[8., 9., 10.], [11., 12., 13.], [14., 15., 16.]],
                1.6,
                1.,
                0.007,
                1.7,
                14.,
                4.782168750,
            ),
            (
                array![[1., 2., 3.], [4., 5., 6.], [7., 8., 9.]],
                array![[8., 9., 10.], [11., 12., 13.], [14., 15., 16.]],
                1.,
                1.6,
                0.003,
                1.5,
                400.,
                4.89790625,
            ),
        ];
        for (u, v, delx, dely, delt, gamma, reynolds, expected) in test_cases {
            assert_eq!(
                calculate_f(
                    ArrayView2::from(&u),
                    ArrayView2::from(&v),
                    delx,
                    dely,
                    delt,
                    gamma,
                    reynolds,
                ),
                expected
            );
        }
    }

    #[test]
    fn test_calculate_g() {
        // These don't have any particular significance, just some random data.
        let test_cases = [
            (
                array![[1., 2., 3.], [4., 5., 6.], [7., 8., 9.]],
                array![[8., 9., 10.], [11., 12., 13.], [14., 15., 16.]],
                1.,
                1.,
                0.005,
                1.7,
                100.,
                11.6825,
            ),
            (
                array![[1., 2., 3.], [4., 5., -6.], [-7., 8., 9.]],
                array![[8., 9., 10.], [11., -12., 13.], [14., 15., -16.]],
                1.,
                1.,
                0.006,
                1.7,
                10.,
                -11.5014,
            ),
            (
                array![[1., 2., 3.], [4., 5., 6.], [7., 8., 9.]],
                array![[8., 9., 10.], [11., 12., 13.], [14., 15., 16.]],
                1.6,
                1.,
                0.007,
                1.7,
                14.,
                11.66141875,
            ),
            (
                array![[1., 2., 3.], [4., 5., 6.], [7., 8., 9.]],
                array![[8., 9., 10.], [11., 12., 13.], [14., 15., 16.]],
                1.,
                1.6,
                0.003,
                1.5,
                400.,
                11.83265625,
            ),
        ];
        for (u, v, delx, dely, delt, gamma, reynolds, expected) in test_cases {
            assert_eq!(
                calculate_g(
                    ArrayView2::from(&u),
                    ArrayView2::from(&v),
                    delx,
                    dely,
                    delt,
                    gamma,
                    reynolds,
                ),
                expected
            );
        }
    }
}
