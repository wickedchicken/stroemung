use ndarray::ArrayView2;

pub type Real = f64;

/// Calculate du^2/dx (the derivative of u^2 over x)
///
/// This function uses the same basic algebra rearrangement that the
/// NaSt2D code does. This makes the function easier to compare against
/// NaSt2D (and reduces the number of divisions in the calculation).
///
/// # Arguments
///
/// * `u_view` - A 3x3-element ArrayView2 representing
///   u[(i-1) to (i+1), (j-1) to (j+1)]. This function only uses
///   the three values on the row where j is 0 (index 1), but takes a 3x3
///   ArrayView2 to be easier to combine with other functions.
/// * `delx` - "delta x," the physical width of the cell
/// * `gamma` - Greek letter gamma, the upwind discretization parameter
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

/// Calculate duv/dx (the derivative of u*v over x)
///
/// This function uses the same basic algebra rearrangement that the
/// NaSt2D code does. This makes the function easier to compare against
/// NaSt2D (and reduces the number of divisions in the calculation).
///
/// # Arguments
///
/// * `u_view` - A 3x3-element ArrayView2 representing
///   u[(i-1) to (i+1), (j-1) to (j+1)]. This function only uses the four
///   "lower left" values (the four combinarions of i-1, i, j, and j+1), but
///   takes a 3x3 ArrayView2 to be easier to combine with other functions.
/// * `v_view` - A 3x3-element ArrayView2 representing
///   v[(i-1) to (i+1), (j-1) to (j+1)]. This function only uses
///   the three values on the row where j is 0 (index 1), but takes a 3x3
///   ArrayView2 to be easier to combine with other functions.
/// * `delx` - "delta x," the physical width of the cell
/// * `gamma` - Greek letter gamma, the upwind discretization parameter
pub fn duvdx(
    u_view: ArrayView2<Real>,
    v_view: ArrayView2<Real>,
    delx: Real,
    gamma: Real,
) -> Real {
    let u_i_j = u_view[(1, 1)]; // u[(i, j)] -> u_i_j
    let u_i_j_p1 = u_view[(1, 2)]; // u[(i, j+1)]  "u[i][j plus 1]" -> u_i_j_p1
    let u_i_m1_j = u_view[(0, 1)]; // "u[(i-1, j)]" "u[i minus 1][j]" -> u_i_m1_j
    let u_i_m1_j_p1 = u_view[(0, 2)]; // "u[(i-1, j+1)]" "u[i-1][j+1]" -> u_i_m1_j_p1

    let v_i_j = v_view[(1, 1)]; // "v[(i, j)]" -> v_i_j
    let v_i_p1_j = v_view[(2, 1)]; // "v[(i+1, j)]" -> "v[i plus 1][j]" -> v_i_p1_j
    let v_i_m1_j = v_view[(0, 1)]; // "v[(i-1, j)]" -> "v[i-1][j]" -> v_i_m1_j

    let inner_left1 = (u_i_j + u_i_j_p1) * (v_i_j + v_i_p1_j);
    let inner_right1 = (u_i_m1_j + u_i_m1_j_p1) * (v_i_m1_j + v_i_j);

    let left_side = inner_left1 - inner_right1;

    let inner_left2 = (u_i_j + u_i_j_p1).abs() * (v_i_j - v_i_p1_j);
    let inner_right2 = (u_i_m1_j + u_i_m1_j_p1).abs() * (v_i_m1_j - v_i_j);

    (left_side + (gamma * (inner_left2 - inner_right2))) / (4.0 * delx)
}

/// Calculate duv/dy (the derivative of u*v over y)
///
/// This function uses the same basic algebra rearrangement that the
/// NaSt2D code does. This makes the function easier to compare against
/// NaSt2D (and reduces the number of divisions in the calculation).
///
/// # Arguments
///
/// * `u_view` - A 3x3-element ArrayView2 representing
///   u[(i-1) to (i+1), (j-1) to (j+1)]. This function only uses
///   the three values on the column where i is 0 (index 1), but takes a 3x3
///   ArrayView2 to be easier to combine with other functions.
/// * `v_view` - A 3x3-element ArrayView2 representing
///   v[(i-1) to (i+1), (j-1) to (j+1)]. This function only uses
///   "upper right" values (the four combinarions of i, i+1, j-1, and j), but
///   takes a 3x3 ArrayView2 to be easier to combine with other functions.
/// * `dely` - "delta y," the physical height of the cell
/// * `gamma` - Greek letter gamma, the upwind discretization parameter
pub fn duvdy(
    u_view: ArrayView2<Real>,
    v_view: ArrayView2<Real>,
    dely: Real,
    gamma: Real,
) -> Real {
    let u_i_j = u_view[(1, 1)]; // u[(i, j)] -> u_i_j
    let u_i_j_m1 = u_view[(1, 0)]; // u[(i, j-1)] -> "u[i][j minus 1]" -> u_i_j_m1
    let u_i_j_p1 = u_view[(1, 2)]; // u[(i, j+1)] -> "u[i][j plus 1]" -> u_i_j_p1

    let v_i_j = v_view[(1, 1)]; // v[(i, j)] -> v_i_j
    let v_i_j_m1 = v_view[(1, 0)]; // v[(i, j-1)] -> "v[i][j minus 1]" -> v_i_j_m1
    let v_i_p1_j = v_view[(2, 1)]; // v[(i+1, j)] -> "v[i plus 1][j]" -> v_i_p1_j
    let v_i_p1_j_m1 = v_view[(2, 0)]; // v[(i+1, j-1)] -> "v[i plus 1][j minus 1]" -> v_i_p1_j_m1

    let inner_left1 = (v_i_j + v_i_p1_j) * (u_i_j + u_i_j_p1);
    let inner_right1 = (v_i_j_m1 + v_i_p1_j_m1) * (u_i_j_m1 + u_i_j);
    let left_side = inner_left1 - inner_right1;

    let inner_left2 = (v_i_j + v_i_p1_j).abs() * (u_i_j - u_i_j_p1);
    let inner_right2 = (v_i_j_m1 + v_i_p1_j_m1).abs() * (u_i_j_m1 - u_i_j);

    (left_side + (gamma * (inner_left2 - inner_right2))) / (4.0 * dely)
}

/// Calculate dv^2/dy (the derivative of v^2 over y)
///
/// This function uses the same basic algebra rearrangement that the
/// NaSt2D code does. This makes the function easier to compare against
/// NaSt2D (and reduces the number of divisions in the calculation).
///
/// # Arguments
///
/// * `v_view` - A 3x3-element ArrayView2 representing
///   v[(i-1) to (i+1), (j-1) to (j+1)]. This function only uses
///   the three values on the column where i is 0 (index 1), but takes a 3x3
///   ArrayView2 to be easier to combine with other functions.
/// * `dely` - "delta y," the physical width of the cell
/// * `gamma` - Greek letter gamma, the upwind discretization parameter
pub fn dv2dy(v_view: ArrayView2<Real>, dely: Real, gamma: Real) -> Real {
    let v_i_j = v_view[(1, 1)]; // v[(i, j)] -> v_i_j
    let v_i_j_p1 = v_view[(1, 2)]; // v[(i, j+1)] -> "v[i][j plus 1]" -> v_i_j_p1
    let v_i_j_m1 = v_view[(1, 0)]; // v[(i, j-1)] -> "v[i][j minus 1]" -> v_i_j_m1

    let inner_left1 = (v_i_j + v_i_j_p1).powi(2);
    let inner_right1 = (v_i_j_m1 + v_i_j).powi(2);

    let left_side = inner_left1 - inner_right1;

    let inner_left2 = (v_i_j + v_i_j_p1).abs() * (v_i_j - v_i_j_p1);
    let inner_right2 = (v_i_j_m1 + v_i_j).abs() * (v_i_j_m1 - v_i_j);

    (left_side + (gamma * (inner_left2 - inner_right2))) / (4.0 * dely)
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
        for (u, delx, gamma, expected) in test_cases {
            assert_eq!(du2dx(ArrayView2::from(&u), delx, gamma), expected);
        }
    }

    #[test]
    fn test_duvdx() {
        // These don't have any particular significance, just some random data.
        let test_cases = [
            (
                array![[1., 2., 3.], [4., 5., 6.], [7., 8., 9.]],
                array![[8., 9., 10.], [11., 12., 13.], [14., 15., 16.]],
                1.,
                1.7,
                40.35,
            ),
            (
                array![[1., 2., 3.], [4., 5., -6.], [-7., 8., 9.]],
                array![[8., 9., 10.], [11., -12., 13.], [14., 15., -16.]],
                1.,
                1.7,
                -53.1,
            ),
            (
                array![[1., 2., 3.], [4., 5., 6.], [7., 8., 9.]],
                array![[8., 9., 10.], [11., 12., 13.], [14., 15., 16.]],
                1.6,
                1.7,
                25.218750,
            ),
            (
                array![[1., 2., 3.], [4., 5., 6.], [7., 8., 9.]],
                array![[8., 9., 10.], [11., 12., 13.], [14., 15., 16.]],
                1.,
                1.5,
                41.25,
            ),
        ];
        for (u, v, delx, gamma, expected) in test_cases {
            assert_eq!(
                duvdx(ArrayView2::from(&u), ArrayView2::from(&v), delx, gamma),
                expected
            );
        }
    }

    #[test]
    fn test_duvdy() {
        // These don't have any particular significance, just some random data.
        let test_cases = [
            (
                array![[1., 2., 3.], [4., 5., 6.], [7., 8., 9.]],
                array![[8., 9., 10.], [11., 12., 13.], [14., 15., 16.]],
                1.,
                1.7,
                17.15,
            ),
            (
                array![[1., 2., 3.], [4., 5., -6.], [-7., 8., 9.]],
                array![[8., 9., 10.], [11., -12., 13.], [14., 15., -16.]],
                1.,
                1.7,
                -32.35,
            ),
            (
                array![[1., 2., 3.], [4., 5., 6.], [7., 8., 9.]],
                array![[8., 9., 10.], [11., 12., 13.], [14., 15., 16.]],
                1.6,
                1.7,
                10.718749999999998,
            ),
            (
                array![[1., 2., 3.], [4., 5., 6.], [7., 8., 9.]],
                array![[8., 9., 10.], [11., 12., 13.], [14., 15., 16.]],
                1.,
                1.5,
                17.25,
            ),
        ];
        for (u, v, dely, gamma, expected) in test_cases {
            assert_eq!(
                duvdy(ArrayView2::from(&u), ArrayView2::from(&v), dely, gamma),
                expected
            );
        }
    }

    #[test]
    fn test_dv2dy() {
        // These don't have any particular significance, just some random data.
        let test_cases = [
            (
                array![[0., 0., 0.], [1., 2., 3.], [0., 0., 0.]],
                1.,
                1.7,
                3.15,
            ),
            (
                array![[0., 0., 0.], [-1., 2., 3.], [0., 0., 0.]],
                1.,
                1.7,
                5.15,
            ),
            (
                array![[0., 0., 0.], [1., 1., 1.], [0., 0., 0.]],
                1.,
                1.7,
                0.,
            ),
            (
                array![[0., 0., 0.], [1., 2., 3.], [0., 0., 0.]],
                1.,
                1.3,
                3.35,
            ),
            (
                array![[0., 0., 0.], [1., 2., 3.], [0., 0., 0.]],
                1.5,
                1.7,
                2.1,
            ),
            (
                array![[0., 0., 0.], [10., 20., 30.], [0., 0., 0.]],
                1.5,
                1.7,
                210.,
            ),
        ];
        for (v, dely, gamma, expected) in test_cases {
            assert_eq!(dv2dy(ArrayView2::from(&v), dely, gamma), expected);
        }
    }
}
