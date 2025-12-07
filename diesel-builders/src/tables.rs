//! Submodule defining and implementing the `Tables` trait.

use diesel::associations::HasTable;

use crate::{Columns, TableAddition, TupleGetColumns, TupleMayGetColumns, Typed};
use tuplities::prelude::*;

/// A trait representing a collection of Diesel tables.
pub trait Tables {
    /// The n-uple of models corresponding to the tables in this collection.
    type Models: TableModels<Tables = Self>
        + TupleLen
        + NestTuple
        + IntoTupleOption<IntoOptions: NestTuple>;
    /// The n-uple of insertable models corresponding to the tables in this
    /// collection.
    type InsertableModels: InsertableTableModels<Tables = Self>
        + TupleLen<Len = <Self::Models as TupleLen>::Len>
        + NestTuple;
    /// Tuple of tuples representing the primary key columns of each table in this collection.
    /// Even non-composite primary keys are represented as single-element tuples for uniformity.
    type PrimaryKeyColumnsCollection: Typed<Type: TupleRefMap + FirstTupleRow>
        + FirstTupleRow<
            FirstRowType: NestTuple<
                Nested: Typed<
                    Type: IntoTupleOption + FlattenNestedTuple<Flattened: IntoTupleOption>,
                >,
            > + Columns<Tables = Self>
                              + Typed<Type: NestTuple + IntoTupleOption<IntoOptions: NestTuple>>,
        > + TupleLen<Len = <Self::Models as TupleLen>::Len>;
}

/// Extension trait for `Tables`.
pub trait TablesExt: Tables<Models: IntoTupleOption<IntoOptions = Self::OptionalModels>> {
    /// The models as an optional tuple.
    type OptionalModels: TupleOption<Transposed = Self::Models> + NestTuple;
}

impl<T> TablesExt for T
where
    T: Tables,
{
    type OptionalModels = <Self::Models as IntoTupleOption>::IntoOptions;
}

/// Trait defining an entity which is associated with a collection of Diesel tables.
pub trait HasTables {
    /// The collection of Diesel tables associated with this entity.
    type Tables: Tables;
}

/// A trait representing a collection of Diesel tables which
/// have non-composite primary keys.
pub trait NonCompositePrimaryKeyTables:
    TablesExt<
        PrimaryKeyColumnsCollection: FirstTupleRow<
            FirstRowType = <Self as NonCompositePrimaryKeyTables>::PrimaryKeys,
        >,
        Models: NestTuple<Nested: TupleGetColumns<<Self::PrimaryKeys as NestTuple>::Nested>>
                    + IntoTupleOption<
            IntoOptions: NestTuple<
                Nested: TupleMayGetColumns<<Self::PrimaryKeys as NestTuple>::Nested>,
            >,
        >,
    >
{
    /// The "flat" n-uple of primary keys, containing the primary key column
    /// for each table in the collection. Since all tables have non-composite
    /// primary keys, this is a flat n-uple.
    type PrimaryKeys: Columns<Tables = Self>
        + Typed<Type: NestTuple + IntoTupleOption<IntoOptions: NestTuple>>
        + NestTuple<
            Nested: Typed<Type: IntoTupleOption + FlattenNestedTuple<Flattened: IntoTupleOption>>,
        >;
}

impl<T> NonCompositePrimaryKeyTables for T
where
    T: TablesExt<
        Models: NestTuple<Nested: TupleGetColumns<<<Self::PrimaryKeyColumnsCollection as FirstTupleRow>::FirstRowType as NestTuple>::Nested>>
            + IntoTupleOption<
                IntoOptions: NestTuple<
                    Nested: TupleMayGetColumns<<<Self::PrimaryKeyColumnsCollection as FirstTupleRow>::FirstRowType as NestTuple>::Nested>,
                >
            >,
    >,
{
    type PrimaryKeys = <Self::PrimaryKeyColumnsCollection as FirstTupleRow>::FirstRowType;
}

/// Trait representing an n-uple of TableModels.
pub trait TableModels: HasTables<Tables: Tables<Models = Self>> + IntoTupleOption {}
impl<T> TableModels for T where T: HasTables<Tables: Tables<Models = T>> + IntoTupleOption {}

/// Trait representing an n-uple of InsertableTableModels.
pub trait InsertableTableModels:
    Sized + TupleDefault + HasTables<Tables: Tables<InsertableModels = Self>>
{
}
impl<T> InsertableTableModels for T where
    T: Sized + TupleDefault + HasTables<Tables: Tables<InsertableModels = T>>
{
}

// Generate implementations for all tuple sizes (0-32)
#[diesel_builders_macros::impl_tables]
mod impls {}
