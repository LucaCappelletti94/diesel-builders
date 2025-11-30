//! Column which is associated to a group of horizontal same-as columns.

use diesel_additions::{HomogeneousColumns, TypedColumn};

use crate::HorizontalSameAsKeys;

/// A trait for Diesel columns that are associated with a group of horizontal
/// same-as columns.
pub trait HorizontalSameAsGroup: TypedColumn {
    /// The group of mandatory horizontal same-as keys associated with this
    /// column.
    type MandatoryHorizontalSameAsKeys: HorizontalSameAsKeys<Self::Table, FirstForeignColumns: HomogeneousColumns<Self::Type>>;
    /// The group of discretionary horizontal same-as keys associated with this
    /// column.
    type DiscretionaryHorizontalSameAsKeys: HorizontalSameAsKeys<Self::Table, FirstForeignColumns: HomogeneousColumns<Self::Type>>;
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
