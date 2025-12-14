//! Column which is associated to a group of vertical same-as columns.

use crate::{
    TypedColumn, ancestors::NestedAncestorColumnsOf, columns::HomogeneouslyTypedNestedColumns,
};

/// A trait for Diesel columns that are associated with a group of vertical
/// same-as columns.
pub trait VerticalSameAsGroup: TypedColumn {
    /// The group of vertical same-as columns associated with this column.
    type VerticalSameAsNestedColumns: HomogeneouslyTypedNestedColumns<Self::ColumnType>
        + NestedAncestorColumnsOf<Self::Table>;
}
