//! Submodule defining and implementing the `NestedTables` trait.

use crate::{HasNestedTables, NestedTableModels, TableExt, columns::NestedColumnsCollection};
use tuplities::prelude::*;

/// Trait for recursive definition of the `Tables` trait.
pub trait NestedTables: FlattenNestedTuple<Flattened: NestTuple> {
    /// The associated nested models.
    type NestedModels: IntoNestedTupleOption<IntoOptions = Self::OptionalNestedModels>
        + FlattenNestedTuple
        + NestedTableModels<NestedTables = Self>;
    /// The associated nested optional models.
    type OptionalNestedModels: NestedTupleOption<Transposed = Self::NestedModels>
        + FlattenNestedTuple
        + HasNestedTables<NestedTables = Self>
        + Default;
    /// The associated nested insertable models.
    type NestedInsertableModels: FlattenNestedTuple;
    /// The associated nested primary key columns collection.
    type NestedPrimaryKeyColumnsCollection: NestedColumnsCollection;
}

impl NestedTables for () {
    type NestedModels = ();
    type OptionalNestedModels = ();
    type NestedInsertableModels = ();
    type NestedPrimaryKeyColumnsCollection = ();
}

impl<T> NestedTables for (T,)
where
    T: TableExt,
    (T::Model,): IntoNestedTupleOption<IntoOptions = (Option<T::Model>,)>,
    (Option<T::Model>,): NestedTupleOption<Transposed = (T::Model,)>,
{
    type NestedModels = (T::Model,);
    type OptionalNestedModels = (Option<T::Model>,);
    type NestedInsertableModels = (T::InsertableModel,);
    type NestedPrimaryKeyColumnsCollection = (T::NestedPrimaryKeyColumns,);
}

impl<Head, Tail> NestedTables for (Head, Tail)
where
    Head: TableExt,
    Tail: NestedTables,
    (Head, Tail): FlattenNestedTuple<Flattened: NestTuple<Nested = (Head, Tail)>>,
    (Head::Model, Tail::NestedModels): NestedTableModels<NestedTables = Self>
        + IntoNestedTupleOption<IntoOptions = (Option<Head::Model>, Tail::OptionalNestedModels)>,
    (Option<Head::Model>, Tail::OptionalNestedModels):
        FlattenNestedTuple + NestedTupleOption<Transposed = (Head::Model, Tail::NestedModels)>,
    (Head::InsertableModel, Tail::NestedInsertableModels): FlattenNestedTuple,
    (
        Head::NestedPrimaryKeyColumns,
        Tail::NestedPrimaryKeyColumnsCollection,
    ): NestedColumnsCollection,
{
    type NestedModels = (Head::Model, Tail::NestedModels);
    type OptionalNestedModels = (Option<Head::Model>, Tail::OptionalNestedModels);
    type NestedInsertableModels = (Head::InsertableModel, Tail::NestedInsertableModels);
    type NestedPrimaryKeyColumnsCollection = (
        Head::NestedPrimaryKeyColumns,
        Tail::NestedPrimaryKeyColumnsCollection,
    );
}
