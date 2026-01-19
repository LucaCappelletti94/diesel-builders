//! Submodule defining and implementing the `NestedColumns` trait.

use std::fmt::Debug;

use tuplities::prelude::{FlattenNestedTuple, NestedTupleIntoVec};

use crate::{DynColumn, TableExt, TypedColumn, TypedNestedTuple};

/// Trait representing a nested tuple of columns.
///
/// Given a tuple of columns `(C1, C2, C3, C4)`, the associated
/// nested columns would be `(C1, (C2, (C3, (C4,))))`.
pub trait NestedColumns: TypedNestedTuple + Default + Copy {
    /// Associated type representing a set of nested tables.
    type NestedTables: FlattenNestedTuple;
    /// The of the columns as a nested tuple of strings.
    type NestedNames: FlattenNestedTuple + Debug + NestedTupleIntoVec<&'static str> + PartialEq + Eq;
    /// Const representing the names of the columns as a nested tuple of
    /// strings.
    const NESTED_COLUMN_NAMES: Self::NestedNames;
    /// Const representing the names of the tables of the columns as a nested
    /// tuple of strings.
    const NESTED_TABLE_NAMES: Self::NestedNames;
}

impl NestedColumns for () {
    type NestedTables = ();
    type NestedNames = ();
    const NESTED_COLUMN_NAMES: Self::NestedNames = ();
    const NESTED_TABLE_NAMES: Self::NestedNames = ();
}

impl<C1: TypedColumn> NestedColumns for (C1,)
where
    C1::Table: TableExt,
{
    type NestedTables = (C1::Table,);
    type NestedNames = (&'static str,);
    const NESTED_COLUMN_NAMES: Self::NestedNames = (C1::NAME,);
    const NESTED_TABLE_NAMES: Self::NestedNames = (C1::Table::TABLE_NAME,);
}

impl<Head, Tail> NestedColumns for (Head, Tail)
where
    Head: TypedColumn,
    Head::Table: TableExt,
    Tail: NestedColumns,
    (Head, Tail): TypedNestedTuple,
    (Head::Table, Tail::NestedTables): FlattenNestedTuple,
    (&'static str, Tail::NestedNames): FlattenNestedTuple,
{
    type NestedTables = (Head::Table, Tail::NestedTables);
    type NestedNames = (&'static str, Tail::NestedNames);
    const NESTED_COLUMN_NAMES: Self::NestedNames = (Head::NAME, Tail::NESTED_COLUMN_NAMES);
    const NESTED_TABLE_NAMES: Self::NestedNames =
        (Head::Table::TABLE_NAME, Tail::NESTED_TABLE_NAMES);
}

/// Trait for n-uples of dynamic columns.
pub trait NestedDynColumns: TypedNestedTuple {
    /// Returns the names of the dynamic columns as a nested tuple of static
    /// strings.
    type NestedDynNames: FlattenNestedTuple
        + Debug
        + NestedTupleIntoVec<&'static str>
        + PartialEq
        + Eq;
    /// Returns the names of the dynamic columns as a nested tuple of static
    /// strings.
    fn nested_dyn_column_names(&self) -> Self::NestedDynNames;
    /// Returns the names of the tables associated with the dynamic columns as a
    /// nested tuple of static strings.
    fn nested_dyn_column_table_names(&self) -> Self::NestedDynNames;
}

impl NestedDynColumns for () {
    type NestedDynNames = ();
    fn nested_dyn_column_names(&self) -> Self::NestedDynNames {}
    fn nested_dyn_column_table_names(&self) -> Self::NestedDynNames {}
}

impl<Head> NestedDynColumns for (DynColumn<Head>,)
where
    Head: 'static + std::fmt::Debug + Clone,
{
    type NestedDynNames = (&'static str,);
    fn nested_dyn_column_names(&self) -> Self::NestedDynNames {
        (self.0.column_name(),)
    }
    fn nested_dyn_column_table_names(&self) -> Self::NestedDynNames {
        (self.0.table_name(),)
    }
}

impl<Head, Tail> NestedDynColumns for (DynColumn<Head>, Tail)
where
    Head: 'static + std::fmt::Debug + Clone,
    Tail: NestedDynColumns,
    (DynColumn<Head>, Tail): TypedNestedTuple,
    (&'static str, Tail::NestedDynNames): FlattenNestedTuple,
{
    type NestedDynNames = (&'static str, Tail::NestedDynNames);
    fn nested_dyn_column_names(&self) -> Self::NestedDynNames {
        (self.0.column_name(), self.1.nested_dyn_column_names())
    }
    fn nested_dyn_column_table_names(&self) -> Self::NestedDynNames {
        (self.0.table_name(), self.1.nested_dyn_column_table_names())
    }
}

/// Trait for types that have dynamic columns.
pub trait HasNestedDynColumns: NestedColumns {
    /// The dynamic columns type.
    type NestedDynColumns: NestedDynColumns<
            NestedTupleValueType = <Self as TypedNestedTuple>::NestedTupleValueType,
            NestedDynNames = <Self as NestedColumns>::NestedNames,
        >;

    /// Returns the dynamic columns as a nested tuple.
    fn nested_dyn_columns() -> Self::NestedDynColumns;
}

impl HasNestedDynColumns for () {
    type NestedDynColumns = ();

    fn nested_dyn_columns() -> Self::NestedDynColumns {}
}

impl<Head> HasNestedDynColumns for (Head,)
where
    Head: TypedColumn<Table: TableExt>,
{
    type NestedDynColumns = (DynColumn<Head::ValueType>,);

    fn nested_dyn_columns() -> Self::NestedDynColumns {
        (Head::default().into(),)
    }
}

impl<Head, Tail> HasNestedDynColumns for (Head, Tail)
where
    Head: TypedColumn<Table: TableExt>,
    Tail: HasNestedDynColumns,
    (Head, Tail): NestedColumns,
    (DynColumn<Head::ValueType>, Tail::NestedDynColumns): NestedDynColumns<
            NestedTupleValueType = Self::NestedTupleValueType,
            NestedDynNames = Self::NestedNames,
        >,
{
    type NestedDynColumns = (DynColumn<Head::ValueType>, Tail::NestedDynColumns);

    fn nested_dyn_columns() -> Self::NestedDynColumns {
        (Head::default().into(), Tail::nested_dyn_columns())
    }
}
