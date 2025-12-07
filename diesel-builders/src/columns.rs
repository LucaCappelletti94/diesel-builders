//! Submodule defining and implementing the `Columns` trait.

use crate::{
    TableAddition, Tables, Typed,
    tables::{InsertableTableModels, TableModels},
};
use tuplities::prelude::*;

#[diesel_builders_macros::impl_columns]
/// A trait representing a collection of Diesel columns.
pub trait Columns:
    TupleDefault + Typed<Type: IntoTupleOption + for<'a> TupleRef<Ref<'a>: IntoTupleOption>>
{
    /// Tables to which these columns belong.
    type Tables: Tables<
            Models: TableModels<Tables = Self::Tables>,
            InsertableModels: InsertableTableModels<Tables = Self::Tables>,
        >;
}

#[diesel_builders_macros::impl_non_empty_projection]
/// A trait representing a non-empty projection of Diesel columns.
pub trait NonEmptyProjection: Columns<Type: TupleRefFront> {
    /// The table associated to this projection.
    type Table: TableAddition;
}
