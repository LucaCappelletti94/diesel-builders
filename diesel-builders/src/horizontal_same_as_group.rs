//! Column which is associated to a group of horizontal same-as columns.

use crate::{
    Columns, HorizontalKeys, HorizontalSameAsNestedKeys, TypedColumn, TypedTuple,
    columns::{HomogeneouslyTypedColumns, HomogeneouslyTypedNestedColumns},
};
use tuplities::prelude::{NestTuple, TupleRow};
use typenum::Unsigned;

/// A trait for Diesel columns that are associated with a group of horizontal
/// same-as columns.
pub trait HorizontalSameAsGroup: TypedColumn {
    /// The index of the column in the host column.
    type Idx: Unsigned;

    /// The group of mandatory horizontal same-as keys associated with this
    /// column.
    type MandatoryHorizontalKeys: HorizontalKeys<
            Self::Table,
            HostColumnsMatrix: TupleRow<Self::Idx, RowType: Columns>,
            ForeignColumnsMatrix: TupleRow<Self::Idx, RowType: Columns + TypedTuple>,
        >;
    /// The group of discretionary horizontal same-as keys associated with this
    /// column.
    type DiscretionaryHorizontalKeys: HorizontalKeys<
            Self::Table,
            HostColumnsMatrix: TupleRow<Self::Idx, RowType: Columns>,
            ForeignColumnsMatrix: TupleRow<Self::Idx, RowType: Columns + TypedTuple>,
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
    /// The mandatory foreign columns associated with this horizontal same-as group.
    type MandatoryForeignColumns: HomogeneouslyTypedColumns<Self::Type, Nested = Self::NestedMandatoryForeignColumns>;
    /// The nested mandatory foreign columns associated with this horizontal same-as group.
    type NestedMandatoryForeignColumns: HomogeneouslyTypedNestedColumns<Self::Type, Flattened = Self::MandatoryForeignColumns>;
    /// The discretionary foreign columns associated with this horizontal same-as group.
    type DiscretionaryForeignColumns: HomogeneouslyTypedColumns<Self::Type, Nested = Self::NestedDiscretionaryForeignColumns>;
    /// The nested discretionary foreign columns associated with this horizontal same-as group.
    type NestedDiscretionaryForeignColumns: HomogeneouslyTypedNestedColumns<Self::Type, Flattened = Self::DiscretionaryForeignColumns>;
    /// The nested mandatory horizontal keys.
    type NestedMandatoryHorizontalKeys: HorizontalSameAsNestedKeys<Self::Table>;
    /// The nested discretionary horizontal keys.
    type NestedDiscretionaryHorizontalKeys: HorizontalSameAsNestedKeys<Self::Table>;
}

impl<T> HorizontalSameAsGroupExt for T
where
    T: HorizontalSameAsGroup,
    <<T::MandatoryHorizontalKeys as HorizontalKeys<
        T::Table,
    >>::ForeignColumnsMatrix as TupleRow<T::Idx>>::RowType: HomogeneouslyTypedColumns<T::Type>,
    <<T::DiscretionaryHorizontalKeys as HorizontalKeys<
        T::Table,
    >>::ForeignColumnsMatrix as TupleRow<T::Idx>>::RowType: HomogeneouslyTypedColumns<T::Type>,
    <<<T::MandatoryHorizontalKeys as HorizontalKeys<
        T::Table,
    >>::ForeignColumnsMatrix as TupleRow<T::Idx>>::RowType as NestTuple>::Nested: HomogeneouslyTypedNestedColumns<T::Type>,
    <<<T::DiscretionaryHorizontalKeys as HorizontalKeys<
        T::Table,
    >>::ForeignColumnsMatrix as TupleRow<T::Idx>>::RowType as NestTuple>::Nested: HomogeneouslyTypedNestedColumns<T::Type>,
    <T::DiscretionaryHorizontalKeys as NestTuple>::Nested: HorizontalSameAsNestedKeys<T::Table>,
    <T::MandatoryHorizontalKeys as NestTuple>::Nested: HorizontalSameAsNestedKeys<T::Table>,
{
    type MandatoryForeignColumns = <<T::MandatoryHorizontalKeys as HorizontalKeys<
        T::Table,
    >>::ForeignColumnsMatrix as TupleRow<T::Idx>>::RowType;
    type DiscretionaryForeignColumns =
        <<T::DiscretionaryHorizontalKeys as HorizontalKeys<
            T::Table,
        >>::ForeignColumnsMatrix as TupleRow<T::Idx>>::RowType;
    type NestedMandatoryForeignColumns = <Self::MandatoryForeignColumns as NestTuple>::Nested;
    type NestedDiscretionaryForeignColumns = <Self::DiscretionaryForeignColumns as NestTuple>::Nested;
    type NestedDiscretionaryHorizontalKeys = <T::DiscretionaryHorizontalKeys as NestTuple>::Nested;
    type NestedMandatoryHorizontalKeys = <T::MandatoryHorizontalKeys as NestTuple>::Nested;
}

/// A marker trait for Diesel columns that are not associated with any group
/// of horizontal same-as columns.
pub trait NoHorizontalSameAsGroup:
    HorizontalSameAsGroup<MandatoryHorizontalKeys = (), DiscretionaryHorizontalKeys = ()>
{
}

impl<T> NoHorizontalSameAsGroup for T where
    T: HorizontalSameAsGroup<MandatoryHorizontalKeys = (), DiscretionaryHorizontalKeys = ()>
{
}
