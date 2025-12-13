//! Column which is associated to a group of horizontal same-as columns.

use crate::{
    Columns, HorizontalNestedKeys, TypedColumn,
    columns::{HomogeneouslyTypedNestedColumns, NestedColumns},
};
use tuplities::prelude::{NestTuple, NestedTupleRow};
use typenum::Unsigned;

/// A trait for Diesel columns that are associated with a group of horizontal
/// same-as columns.
pub trait HorizontalSameAsGroup: TypedColumn {
    /// The index of the column in the host column.
    type Idx: Unsigned;

    /// The group of mandatory horizontal same-as keys associated with this
    /// column.
    type MandatoryHorizontalKeys: Columns<
        Nested: HorizontalNestedKeys<
            Self::Table,
            NestedHostColumnsMatrix: NestedTupleRow<Self::Idx, RowType: NestedColumns>,
            NestedForeignColumnsMatrix: NestedTupleRow<Self::Idx, RowType: NestedColumns>,
        >,
    >;
    /// The group of discretionary horizontal same-as keys associated with this
    /// column.
    type DiscretionaryHorizontalKeys: Columns<
        Nested: HorizontalNestedKeys<
            Self::Table,
            NestedHostColumnsMatrix: NestedTupleRow<Self::Idx, RowType: NestedColumns>,
            NestedForeignColumnsMatrix: NestedTupleRow<Self::Idx, RowType: NestedColumns>,
        >,
    >;
}

/// Extension trait for `HorizontalSameAsGroup` to provide associated types
/// for mandatory and discretionary host and foreign columns.
pub trait HorizontalSameAsGroupExt:
    HorizontalSameAsGroup<
        MandatoryHorizontalKeys: NestTuple<Nested = Self::NestedMandatoryHorizontalKeys>,
        DiscretionaryHorizontalKeys: NestTuple<Nested = Self::NestedDiscretionaryHorizontalKeys>,
    >
{
    /// The nested mandatory foreign columns associated with this horizontal same-as group.
    type NestedMandatoryForeignColumns: HomogeneouslyTypedNestedColumns<Self::Type>;
    /// The nested discretionary foreign columns associated with this horizontal same-as group.
    type NestedDiscretionaryForeignColumns: HomogeneouslyTypedNestedColumns<Self::Type>;
    /// The nested mandatory horizontal keys.
    type NestedMandatoryHorizontalKeys: HorizontalNestedKeys<Self::Table>;
    /// The nested discretionary horizontal keys.
    type NestedDiscretionaryHorizontalKeys: HorizontalNestedKeys<Self::Table>;
}

impl<T> HorizontalSameAsGroupExt for T
where
    T: HorizontalSameAsGroup,
    <<<T::MandatoryHorizontalKeys as NestTuple>::Nested as HorizontalNestedKeys<
        T::Table,
    >>::NestedForeignColumnsMatrix as NestedTupleRow<T::Idx>>::RowType: HomogeneouslyTypedNestedColumns<T::Type>,
    <<<T::DiscretionaryHorizontalKeys as NestTuple>::Nested as HorizontalNestedKeys<
        T::Table,
    >>::NestedForeignColumnsMatrix as NestedTupleRow<T::Idx>>::RowType: HomogeneouslyTypedNestedColumns<T::Type>,
    <T::DiscretionaryHorizontalKeys as NestTuple>::Nested: HorizontalNestedKeys<T::Table>,
    <T::MandatoryHorizontalKeys as NestTuple>::Nested: HorizontalNestedKeys<T::Table>,
{
    type NestedMandatoryForeignColumns = <<<T::MandatoryHorizontalKeys as NestTuple>::Nested as HorizontalNestedKeys<
        T::Table,
    >>::NestedForeignColumnsMatrix as NestedTupleRow<T::Idx>>::RowType;
    type NestedDiscretionaryForeignColumns = <<<T::DiscretionaryHorizontalKeys as NestTuple>::Nested as HorizontalNestedKeys<
        T::Table,
    >>::NestedForeignColumnsMatrix as NestedTupleRow<T::Idx>>::RowType;
    type NestedDiscretionaryHorizontalKeys = <T::DiscretionaryHorizontalKeys as NestTuple>::Nested;
    type NestedMandatoryHorizontalKeys = <T::MandatoryHorizontalKeys as NestTuple>::Nested;
}
