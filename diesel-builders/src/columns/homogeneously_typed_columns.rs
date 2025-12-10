//! Submodule defining and implementing the `HomogeneouslyTypedColumns` trait.

use crate::HomogeneouslyTypedTuple;

use super::Columns;

/// A trait representing a tuple of columns where all columns have the same associated Type.
pub trait HomogeneouslyTypedColumns<CT>: Columns + HomogeneouslyTypedTuple<CT> {}

impl<T, CT> HomogeneouslyTypedColumns<CT> for T where T: Columns + HomogeneouslyTypedTuple<CT> {}
