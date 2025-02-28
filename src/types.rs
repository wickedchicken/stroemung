use crate::math::Real;
use ndarray::{Array, Ix2};

pub type Velocity = [Real; 2];

pub type GridArray<T> = Array<T, Ix2>;

pub type GridSize = [usize; 2];

// According to https://docs.rs/ndarray/0.16.1/ndarray/struct.ArrayBase.html,
// "The default memory order of an array is row major order (a.k.a “c” order),
// where each row is contiguous in memory." This means we can use the default
// implementation of PartialOrd and Ord. If we were using column-major ordering,
// we would need to implement an ordering that prioritized the y value over the
// x value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BoundaryIndex(pub usize, pub usize);

// It would be nice to unify BoundaryIndex and GridIndex into one type that
// can be sorted and also directly used by ndarray's indexing operations.
pub type GridIndex = (usize, usize);
