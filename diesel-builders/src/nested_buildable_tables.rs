//! Module for buildable columns in Diesel.

use tuplities::prelude::*;

use crate::{BuildableTable, HasNestedTables, TableBuilder, tables::NestedTables};

/// A trait for nested Diesel tables that have associated builders.
pub trait NestedBuildableTables: NestedTables {
    /// The builders associated with the buildable tables.
    type NestedBuilders: IntoNestedTupleOption<IntoOptions = Self::NestedOptionalBuilders>
        + FlattenNestedTuple<Flattened: IntoTupleOption>
        + HasNestedTables<NestedTables = Self>
        + Default;
    /// The optional builders associated with the buildable tables.
    type NestedOptionalBuilders: NestedTupleOption<Transposed = Self::NestedBuilders>
        + FlattenNestedTuple<Flattened: TupleOption>
        + HasNestedTables<NestedTables = Self>
        + Default;
}

impl NestedBuildableTables for () {
    type NestedBuilders = ();
    type NestedOptionalBuilders = ();
}

impl<T1> NestedBuildableTables for (T1,)
where
    T1: BuildableTable,
{
    type NestedBuilders = (TableBuilder<T1>,);
    type NestedOptionalBuilders = (Option<TableBuilder<T1>>,);
}

impl<Thead, Ttail> NestedBuildableTables for (Thead, Ttail)
where
    Thead: BuildableTable,
    Ttail: NestedBuildableTables,
    (Thead, Ttail): NestedTables,
    (TableBuilder<Thead>, Ttail::NestedBuilders): FlattenNestedTuple<Flattened: IntoTupleOption>,
    (Option<TableBuilder<Thead>>, Ttail::NestedOptionalBuilders):
        FlattenNestedTuple<Flattened: TupleOption>,
{
    type NestedBuilders = (TableBuilder<Thead>, Ttail::NestedBuilders);
    type NestedOptionalBuilders = (Option<TableBuilder<Thead>>, Ttail::NestedOptionalBuilders);
}
