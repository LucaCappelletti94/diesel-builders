//! Column which is associated to a group of horizontal same-as columns.

use crate::{Columns, HorizontalSameAsKeys, Typed, TypedColumn};
use tuplities::prelude::{TupleReplicate, TupleRow};
use typenum::Unsigned;

/// A trait for Diesel columns that are associated with a group of horizontal
/// same-as columns.
pub trait HorizontalSameAsGroup: TypedColumn {
    /// The index of the column in the host column.
    type Idx: Unsigned;

    /// The group of mandatory horizontal same-as keys associated with this
    /// column.
    type MandatoryHorizontalSameAsKeys: HorizontalSameAsKeys<
            Self::Table,
            HostColumnsMatrix: TupleRow<Self::Idx, RowType: Columns + TupleReplicate<Self>>,
            ForeignColumnsMatrix: TupleRow<
                Self::Idx,
                RowType: Columns + Typed<Type: TupleReplicate<Self::Type>>,
            >,
        >;
    /// The group of discretionary horizontal same-as keys associated with this
    /// column.
    type DiscretionaryHorizontalSameAsKeys: HorizontalSameAsKeys<
            Self::Table,
            HostColumnsMatrix: TupleRow<Self::Idx, RowType: Columns + TupleReplicate<Self>>,
            ForeignColumnsMatrix: TupleRow<
                Self::Idx,
                RowType: Columns + Typed<Type: TupleReplicate<Self::Type>>,
            >,
        >;
}

/// Extension trait for `HorizontalSameAsGroup` to provide associated types
/// for mandatory and discretionary host and foreign columns.
pub trait HorizontalSameAsGroupExt: HorizontalSameAsGroup {
    /// The mandatory foreign columns associated with this horizontal same-as group.
    type MandatoryForeignColumns: Columns<Type: TupleReplicate<Self::Type>>;
    /// The discretionary foreign columns associated with this horizontal same-as group.
    type DiscretionaryForeignColumns: Columns<Type: TupleReplicate<Self::Type>>;
}

impl<T> HorizontalSameAsGroupExt for T
where
    T: HorizontalSameAsGroup,
{
    type MandatoryForeignColumns = <<T::MandatoryHorizontalSameAsKeys as HorizontalSameAsKeys<
        T::Table,
    >>::ForeignColumnsMatrix as TupleRow<T::Idx>>::RowType;
    type DiscretionaryForeignColumns =
        <<T::DiscretionaryHorizontalSameAsKeys as HorizontalSameAsKeys<
            T::Table,
        >>::ForeignColumnsMatrix as TupleRow<T::Idx>>::RowType;
}

/// A marker trait for Diesel columns that are not associated with any group
/// of horizontal same-as columns.
pub trait NoHorizontalSameAsGroup:
    HorizontalSameAsGroup<MandatoryHorizontalSameAsKeys = (), DiscretionaryHorizontalSameAsKeys = ()>
{
}

impl<T> NoHorizontalSameAsGroup for T where
    T: HorizontalSameAsGroup<
            MandatoryHorizontalSameAsKeys = (),
            DiscretionaryHorizontalSameAsKeys = (),
        >
{
}
