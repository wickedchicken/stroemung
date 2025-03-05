use std::fmt;
use std::io::Read;

use crate::cell::Cell;
use crate::math::Real;
use crate::math::{du2dx, duvdx, duvdy, dv2dy, laplacian, residual};

use serde::Deserialize;
use serde::Serialize;

use serde_json::Error as SerdeError;

use thiserror::Error;

use crate::grid::{
    EdgeType, SimulationGrid, SimulationGridError, UnfinalizedSimulationGrid,
};
use crate::types::{CellPhysicalSize, GridArray, GridSize};

use ndarray::{s, Array, ArrayView2, Zip};

#[derive(Error, Debug)]
pub enum SimulationError {
    #[error("An error occurred while deserializing: `{0}`")]
    DeserializationError(#[from] SerdeError),
    #[error("An error occurred with the SimulationGrid: `{0}`")]
    GridError(#[from] SimulationGridError),
}

#[derive(Debug, Deserialize)]
pub struct UnfinalizedSimulation {
    pub size: GridSize,
    pub cell_size: CellPhysicalSize,
    pub delt: Real,
    pub gamma: Real,
    pub reynolds: Real,
    pub initial_norm_squared: Option<Real>,
    pub sor_absolute_epsilon: Real,
    pub max_iterations: u32,
    pub iterations: u32,
    pub time: Real,
    pub omega: Real,
    pub grid: UnfinalizedSimulationGrid,
}

// This must be the same as UnfinalizedSimulation, except the type
// of grid and without the calculated values. We have two types to make sure
// we never deserialize without forgetting to generate the boundary list.
#[derive(Debug, Serialize)]
pub struct Simulation {
    pub size: GridSize,
    pub cell_size: CellPhysicalSize,
    pub delt: Real,
    pub gamma: Real,
    pub reynolds: Real,
    #[serde(skip)]
    pub f: GridArray<Real>,
    #[serde(skip)]
    pub g: GridArray<Real>,
    #[serde(skip)]
    pub rhs: GridArray<Real>,
    pub initial_norm_squared: Option<Real>,
    pub sor_absolute_epsilon: Real,
    pub max_iterations: u32,
    pub iterations: u32,
    pub time: Real,
    pub omega: Real,
    pub grid: SimulationGrid,
}

impl TryFrom<UnfinalizedSimulation> for Simulation {
    type Error = SimulationError;

    fn try_from(item: UnfinalizedSimulation) -> Result<Self, Self::Error> {
        // Will be nicer once https://github.com/rust-lang/rust/issues/86555
        // is in stable.
        let mut sim = Simulation {
            size: item.size,
            cell_size: item.cell_size,
            delt: item.delt,
            gamma: item.gamma,
            reynolds: item.reynolds,
            f: Array::zeros(item.size),
            g: Array::zeros(item.size),
            rhs: Array::zeros(item.size),
            initial_norm_squared: item.initial_norm_squared,
            sor_absolute_epsilon: item.sor_absolute_epsilon,
            max_iterations: item.max_iterations,
            iterations: item.iterations,
            time: item.time,
            omega: item.omega,
            grid: item.grid.try_into()?,
        };
        sim.calculate_f_and_g();
        sim.calculate_rhs();
        sim.get_initial_norm_squared();
        Ok(sim)
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
    pub fn from_reader<R: Read>(reader: R) -> Result<Simulation, SimulationError> {
        let unfinalized: UnfinalizedSimulation = serde_json::from_reader(reader)?;
        Simulation::try_from(unfinalized)
    }

    fn calculate_f_and_g(&mut self) {
        // Ignore outer boundary. This also gives us the correct shape, because
        // everything is computed using 3x3 grids which aren't defined on the
        // boundary.
        // Clippy doesn't like ndarray slicing with negative indices. See
        // https://github.com/rust-lang/rust-clippy/issues/5808 and
        // https://github.com/rust-ndarray/ndarray/pull/1279 for details.
        #[allow(clippy::reversed_empty_ranges)]
        let mut f_window = self.f.slice_mut(s![1..-1, 1..-1]);
        #[allow(clippy::reversed_empty_ranges)]
        let mut g_window = self.g.slice_mut(s![1..-1, 1..-1]);

        // ndarray doesn't have masked arrays. To avoid an if statement inside
        // a core loop, we compute F and G over everything and postprocess the
        // boundaries afterward. It would be good to benchmark if this is
        // actually helpful or not.
        Zip::from(&mut f_window)
            .and(&mut g_window)
            .and(self.grid.u.windows((3, 3)))
            .and(self.grid.v.windows((3, 3)))
            .for_each(|f, g, u_view, v_view| {
                *f = calculate_f(
                    u_view,
                    v_view,
                    self.cell_size[0],
                    self.cell_size[1],
                    self.delt,
                    self.gamma,
                    self.reynolds,
                );
                *g = calculate_g(
                    u_view,
                    v_view,
                    self.cell_size[0],
                    self.cell_size[1],
                    self.delt,
                    self.gamma,
                    self.reynolds,
                );
            });

        // Restore F and G on boundary edges, where they shouldn't have been
        // updated
        // Todo: maybe restore with a fixed save list like
        // self.grid.boundaries.u_v_restore
        for (boundary_idx, maybe_edge) in &self.grid.boundaries.sorted_boundary_list {
            self.f[*boundary_idx] = self.grid.u[*boundary_idx];
            self.g[*boundary_idx] = self.grid.v[*boundary_idx];

            // North and west boundaries also need their corresponding fluid
            // neighbors updated.
            match maybe_edge {
                Some(EdgeType::North { north_neighbor }) => {
                    self.g[*north_neighbor] = self.grid.v[*north_neighbor];
                }
                Some(EdgeType::NorthWest {
                    north_neighbor,
                    west_neighbor,
                }) => {
                    self.f[*west_neighbor] = self.grid.u[*west_neighbor];
                    self.g[*north_neighbor] = self.grid.v[*north_neighbor];
                }
                Some(EdgeType::West { west_neighbor }) => {
                    self.f[*west_neighbor] = self.grid.u[*west_neighbor];
                }
                Some(EdgeType::SouthWest {
                    south_neighbor: _,
                    west_neighbor,
                }) => {
                    self.f[*west_neighbor] = self.grid.u[*west_neighbor];
                }
                Some(EdgeType::NorthEast {
                    north_neighbor,
                    east_neighbor: _,
                }) => {
                    self.g[*north_neighbor] = self.grid.v[*north_neighbor];
                }
                None | Some(_) => {}
            }
        }
    }

    fn calculate_rhs(&mut self) {
        let mut rhs_view = self.rhs.slice_mut(s![1.., 1..]);
        Zip::from(&mut rhs_view)
            .and(self.f.windows((2, 2)))
            .and(self.g.windows((2, 2)))
            .for_each(|rhs, f_view, g_view| {
                *rhs = (((f_view[(1, 1)] - f_view[(0, 1)]) / self.cell_size[0])
                    + ((g_view[(1, 1)] - g_view[(1, 0)]) / self.cell_size[1]))
                    / self.delt
            });
    }

    fn calculate_norm_squared(&self) -> Real {
        #[allow(clippy::reversed_empty_ranges)]
        let rhses = self.rhs.slice(s![1..-1, 1..-1]);

        let sums = Zip::from(self.grid.pressure.windows((3, 3)))
            .and(rhses)
            .fold(0.0, |acc, p_view, rhs| {
                acc + residual(p_view, self.cell_size[0], self.cell_size[1], *rhs).powi(2)
            });

        sums / self.grid.boundaries.fluid_cells
    }

    fn get_initial_norm_squared(&mut self) -> Real {
        if let Some(norm) = self.initial_norm_squared {
            return norm;
        }

        let norm = self.calculate_norm_squared();
        self.initial_norm_squared = Some(norm);
        norm
    }

    fn solve_sor(&mut self) -> Result<(u32, Real), SimulationGridError> {
        let delx2 = self.cell_size[0].powi(2);
        let dely2 = self.cell_size[1].powi(2);

        let one_minus_w = 1.0 - self.omega;
        let middle = self.omega / ((2.0 / delx2) + (2.0 / dely2));

        let epsilon_squared = self.sor_absolute_epsilon.powi(2);

        let mut norm_squared = 0.0;

        for i in 0..self.max_iterations {
            self.grid.copy_pressure_to_boundaries()?;
            // indexing instead of iterators :(
            for x in 1..self.size[0] - 1 {
                // indexing instead of iterators :(
                for y in 1..self.size[1] - 1 {
                    // if statement in inner loop :(
                    if let Cell::Fluid = self.grid.cell_type[(x, y)] {
                        // Note that we're modifying in place, so "minus one"
                        // values have been computed for the next step already.
                        let p_i_j = self.grid.pressure[(x, y)];
                        let p_i_m1_j = self.grid.pressure[(x - 1, y)];
                        let p_i_p1_j = self.grid.pressure[(x + 1, y)];
                        let p_i_j_m1 = self.grid.pressure[(x, y - 1)];
                        let p_i_j_p1 = self.grid.pressure[(x, y + 1)];
                        let rhs = self.rhs[(x, y)];

                        self.grid.pressure[(x, y)] = (one_minus_w * p_i_j)
                            + middle
                                * (((p_i_p1_j + p_i_m1_j) / delx2)
                                    + ((p_i_j_p1 + p_i_j_m1) / dely2)
                                    - rhs)
                    }
                }
            }

            let initial_norm_squared = self.get_initial_norm_squared();
            norm_squared = self.calculate_norm_squared();

            if (norm_squared < initial_norm_squared) || (norm_squared < epsilon_squared) {
                return Ok((i + 1, norm_squared));
            }
        }
        Ok((self.max_iterations, norm_squared))
    }

    pub fn set_u_and_v(&mut self) {
        #[allow(clippy::reversed_empty_ranges)]
        let mut u_view = self.grid.u.slice_mut(s![0..-1, 0..-1]);
        #[allow(clippy::reversed_empty_ranges)]
        let mut v_view = self.grid.v.slice_mut(s![0..-1, 0..-1]);

        #[allow(clippy::reversed_empty_ranges)]
        let f_view = self.f.slice_mut(s![0..-1, 0..-1]);

        #[allow(clippy::reversed_empty_ranges)]
        let g_view = self.g.slice_mut(s![0..-1, 0..-1]);

        Zip::from(&mut u_view)
            .and(&mut v_view)
            .and(self.grid.pressure.windows((2, 2)))
            .and(f_view)
            .and(g_view)
            .for_each(|u, v, p_view, f, g| {
                let p_i_j = p_view[(0, 0)];
                let p_i_p1_j = p_view[(1, 0)];
                let p_i_j_p1 = p_view[(0, 1)];

                *u = *f - (self.delt / self.cell_size[0]) * (p_i_p1_j - p_i_j);
                *v = *g - (self.delt / self.cell_size[1]) * (p_i_j_p1 - p_i_j);
            });

        for (idx, maybe_u, maybe_v) in &self.grid.boundaries.u_v_restore {
            if let Some(u) = maybe_u {
                self.grid.u[*idx] = *u;
            }
            if let Some(v) = maybe_v {
                self.grid.v[*idx] = *v;
            }
        }
    }

    pub fn run_simulation_tick(&mut self) -> Result<(u32, Real), SimulationError> {
        self.grid.set_boundary_u_and_v()?;
        self.calculate_f_and_g();
        self.calculate_rhs();
        let (sor_iterations, norm_squared) = self.solve_sor()?;
        self.set_u_and_v();
        self.time += self.delt;
        self.iterations += 1;
        Ok((sor_iterations, norm_squared))
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
        for test_filename in [
            test_data_directory().join("simple_simulation.json"),
            test_data_directory().join("small_simulation_with_boundaries.json"),
        ] {
            let result = Simulation::from_reader(BufReader::new(
                File::open(test_filename).unwrap(),
            ))
            .unwrap();
            insta::assert_json_snapshot!(result);
        }
    }

    #[test]
    fn serialize() {
        let size = [5, 7];
        let cell_size = [1., 2.];
        let delt = 1.4;
        let gamma = 1.7;
        let reynolds = 100.;

        let simulation = Simulation::try_from(UnfinalizedSimulation {
            size,
            cell_size,
            delt,
            gamma,
            reynolds,
            initial_norm_squared: Default::default(),
            sor_absolute_epsilon: 0.001,
            max_iterations: 100,
            iterations: 0,
            time: 0.0,
            omega: 1.7,
            grid: presets::empty(size).into(),
        })
        .unwrap();

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

    #[test]
    fn simulation_tick() {
        let size = [4, 3];
        let mut sim = Simulation::try_from(UnfinalizedSimulation {
            size,
            cell_size: [0.1, 0.2],
            delt: 0.005,
            gamma: 0.9,
            reynolds: 100.0,
            sor_absolute_epsilon: 0.001,
            max_iterations: 100,
            initial_norm_squared: None,
            iterations: 0,
            time: 0.0,
            omega: 1.7,
            grid: presets::simple_inflow(size).into(),
        })
        .unwrap();

        let (sor_iterations, norm_squared) = sim.run_simulation_tick().unwrap();
        insta::assert_json_snapshot!(sim.f);
        insta::assert_json_snapshot!(sim.g);
        insta::assert_json_snapshot!(sim.rhs);
        insta::assert_json_snapshot!(sim);
        // SOR is bad at converging on "unphysical" initial conditions, hence
        // the first few ticks are expected to stop after max_iterations.
        assert_eq!(sor_iterations, 100);
        assert_eq!(norm_squared, 562901.7447199143);

        let mut last_sor_iterations = 0;
        let mut last_norm_squared = 0.0;
        for _ in 0..100 {
            (last_sor_iterations, last_norm_squared) = sim.run_simulation_tick().unwrap();
        }
        assert_eq!(last_sor_iterations, 1);
        assert_eq!(last_norm_squared, 3.8344148218167323e-20);
        insta::assert_json_snapshot!(sim.f);
        insta::assert_json_snapshot!(sim.g);
        insta::assert_json_snapshot!(sim.rhs);
        insta::assert_json_snapshot!(sim);

        for _ in 0..100 {
            sim.run_simulation_tick().unwrap();
        }
        // We're interested to see if the pressure and velocity
        // stay stable after 100 iterations
        insta::assert_json_snapshot!(sim);
    }
}
