use crate::math::Real;
use crate::math::{du2dx, duvdx, duvdy, dv2dy, laplacian};

use ndarray::ArrayView2;

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
