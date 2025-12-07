//! Submodule defining and implementing the `Tables` trait.

use diesel::associations::HasTable;

use crate::{Columns, TableAddition, TupleMayGetColumns, Typed, get_set_columns::TupleGetColumns};
use tuplities::prelude::*;

/// A trait representing a collection of Diesel tables.
pub trait Tables {
    /// The n-uple of models corresponding to the tables in this collection.
    type Models: TableModels<Tables = Self> + TupleLen;
    /// The n-uple of insertable models corresponding to the tables in this
    /// collection.
    type InsertableModels: InsertableTableModels<Tables = Self>
        + TupleLen<Len = <Self::Models as TupleLen>::Len>;
    /// Tuple of tuples representing the primary key columns of each table in this collection.
    /// Even non-composite primary keys are represented as single-element tuples for uniformity.
    type PrimaryKeyColumnsCollection: Typed<Type: TupleRefMap + FirstTupleRow>
        + FirstTupleRow<FirstRowType: Columns<Tables = Self>>
        + TupleLen<Len = <Self::Models as TupleLen>::Len>;
}

/// Extension trait for `Tables`.
pub trait TablesExt: Tables<Models: IntoTupleOption<IntoOptions = Self::OptionalModels>> {
    /// The models as an optional tuple.
    type OptionalModels: TupleOption<Transposed = Self::Models>;
}

impl<T> TablesExt for T
where
    T: Tables<Models: IntoTupleOption>,
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
        Models: TupleGetColumns<<Self as NonCompositePrimaryKeyTables>::PrimaryKeys>,
        OptionalModels: TupleMayGetColumns<<Self as NonCompositePrimaryKeyTables>::PrimaryKeys>,
    >
{
    /// The "flat" n-uple of primary keys, containing the primary key column
    /// for each table in the collection. Since all tables have non-composite
    /// primary keys, this is a flat n-uple.
    type PrimaryKeys: Columns<Tables = Self>;
}

impl<T> NonCompositePrimaryKeyTables for T
where
    T: TablesExt<
            Models: TupleGetColumns<
                <Self::PrimaryKeyColumnsCollection as FirstTupleRow>::FirstRowType,
            >,
            OptionalModels: TupleMayGetColumns<
                <Self::PrimaryKeyColumnsCollection as FirstTupleRow>::FirstRowType,
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
