use ndarray::ArrayView2;

pub type Real = f64;

/// Calculates du^2/dx (the derivative of u^2 over x)
///
/// This function uses the same basic algebra rearrangement that the
/// NaSt2D code does. This makes the function easier to compare against
/// NaSt2D (and reduces the number of divisions in the calculation).
///
/// # Arguments
///
/// * `u_view` - A 3x3-element ArrayView2 representing
///   u[(i-1) to (i+1), (j-1) to (j+1)]. This function only uses
///   the three values on the column where j is 0 (index 1), but takes a 3x3
///   ArrayView2 to be easier to combine with other functions.
/// * `gamma` - Greek letter gamma, the upwind discretization parameter
/// * `delx` - "delta x," the physical width of the cell
pub fn du2dx(u_view: ArrayView2<Real>, delx: Real, gamma: Real) -> Real {
    let u_i_m1 = u_view[(0, 1)]; // u[(i-1, j)]  "u[i minus 1]" -> u_i_m1
    let u_i = u_view[(1, 1)]; // u[(i, j)]  "u[i]" -> "u_i"
    let u_i_p1 = u_view[(2, 1)]; // u[(i+1, j)]  "u[i plus 1]" -> u_i_p1

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
    use ndarray::{array, ArrayView2};

    #[test]
    fn test_du2dx() {
        // These don't have any particular significance, just some random data.
        let test_cases = [
            (
                array![[0., 1., 0.], [0., 2., 0.], [0., 3., 0.]],
                1.,
                1.7,
                3.15,
            ),
            (
                array![[0., -1., 0.], [0., 2., 0.], [0., 3., 0.]],
                1.,
                1.7,
                5.15,
            ),
            (
                array![[0., 1., 0.], [0., 1., 0.], [0., 1., 0.]],
                1.,
                1.7,
                0.,
            ),
            (
                array![[0., 1., 0.], [0., 2., 0.], [0., 3., 0.]],
                1.,
                1.3,
                3.35,
            ),
            (
                array![[0., 1., 0.], [0., 2., 0.], [0., 3., 0.]],
                1.5,
                1.7,
                2.1,
            ),
            (
                array![[0., 10., 0.], [0., 20., 0.], [0., 30., 0.]],
                1.5,
                1.7,
                210.,
            ),
        ];
        for (u, gamma, delx, expected) in test_cases {
            assert_eq!(du2dx(ArrayView2::from(&u), gamma, delx), expected);
        }
    }
}
