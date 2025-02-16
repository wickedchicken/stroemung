use crate::math::Real;
use ndarray::{Array, Ix2};

pub type Velocity = [Real; 2];

pub type GridArray<T> = Array<T, Ix2>;

pub type GridSize = [usize; 2];
