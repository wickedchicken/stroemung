use ndarray::ArrayView1;

pub type Real = f64;

/// Calculates du^2/dx (the derivative of u^2 over x)
///
/// This function uses the same basic algebra rearrangement that the
/// NaSt2D code does. This makes the function easier to compare against
/// NaSt2D (and reduces the number of divisions in the calculation).
///
/// # Arguments
///
/// * `u_view` - A 3-element ArrayView1 representing u[(i-1), j],  u[(i, j)]
///   and u[(i+1, j)]. Since all three elements are on the same axis, the j
///   index is unnecessary and the i index is relative: [-1, 0, 1].
/// * `gamma` - Greek letter gamma, the upwind discretization parameter
/// * `delx` - "delta x," the physical width of the cell
pub fn du2dx_kernel(u_view: ArrayView1<Real>, delx: Real, gamma: Real) -> Real {
    let u_i_m1 = u_view[0]; // u[(i-1, j)]  "u[i minus 1]" -> u_i_m1
    let u_i = u_view[1]; // u[(i, j)]  "u[i]" -> "u_i"
    let u_i_p1 = u_view[2]; // u[(i+1, j)]  "u[i plus 1]" -> u_i_p1

    let inner_left1 = (u_i + u_i_p1).powi(2);
    let inner_right1 = (u_i_m1 + u_i).powi(2);

    let left_side = inner_left1 - inner_right1;

    let inner_left2 = (u_i + u_i_p1).abs() * (u_i - u_i_p1);
    let inner_right2 = (u_i_m1 + u_i).abs() * (u_i_m1 - u_i);

    (left_side + (gamma * (inner_left2 - inner_right2))) / (4.0 * delx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::{array, ArrayView1};

    #[test]
    fn test_du2dx_kernel() {
        // These don't have any particular significance, just some random data.
        let test_cases = [
            (array![1., 2., 3.], 1., 1.7, 3.15),
            (array![-1., 2., 3.], 1., 1.7, 5.15),
            (array![1., 1., 1.], 1., 1.7, 0.),
            (array![1., 2., 3.], 1., 1.3, 3.35),
            (array![1., 2., 3.], 1.5, 1.7, 2.1),
            (array![10., 20., 30.], 1.5, 1.7, 210.),
        ];
        for (u, gamma, delx, expected) in test_cases {
            assert_eq!(du2dx_kernel(ArrayView1::from(&u), gamma, delx), expected);
        }
    }
}
