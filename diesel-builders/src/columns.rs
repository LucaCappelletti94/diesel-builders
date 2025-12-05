//! Submodule defining and implementing the `Columns` trait.

use crate::{
    TableAddition, Tables, TypedColumn,
    tables::{InsertableTableModels, TableModels},
};
use tuplities::prelude::*;

#[diesel_builders_macros::impl_columns]
/// A trait representing a collection of Diesel columns.
pub trait Columns: TupleDefault {
    /// Tuple of data types of the columns.
    type Types: IntoTupleOption + for<'a> TupleRef<Ref<'a>: IntoTupleOption>;
    /// Tables to which these columns belong.
    type Tables: Tables<
            Models: TableModels<Tables = Self::Tables>,
            InsertableModels: InsertableTableModels<Tables = Self::Tables>,
        >;
}

#[diesel_builders_macros::impl_projection]
/// A trait representing a potentially empty projection of Diesel columns.
pub trait Projection<T: TableAddition>: Columns {}

/// A trait representing a non-empty projection of Diesel columns.
pub trait NonEmptyProjection: Projection<Self::Table> {
    /// The table associated to this projection.
    type Table: TableAddition;
}

impl<T> NonEmptyProjection for T
where
    T: TuplePopFront<Front: diesel::Column<Table: TableAddition>>,
    T: Projection<<<T as TuplePopFront>::Front as diesel::Column>::Table>,
{
    type Table = <<T as TuplePopFront>::Front as diesel::Column>::Table;
}

#[diesel_builders_macros::impl_homogeneous_columns]
/// A trait representing a collection of Diesel columns with an associated type.
pub trait HomogeneousColumns<Type>: Columns {}
