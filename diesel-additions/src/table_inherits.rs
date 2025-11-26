//! Submodule implementing the `TableInherits` trait for Diesel tables.

use crate::{SingletonForeignKey, TypedColumn, table_addition::HasPrimaryKey};

/// A trait for Diesel tables that inherit from other tables.
pub trait TableInherits<Other: HasPrimaryKey>:
    HasPrimaryKey<
    PrimaryKey: SingletonForeignKey<ReferencedTable = Other>
                    + TypedColumn<Type = <Other::PrimaryKey as TypedColumn>::Type>,
>
{
}

impl<Other, T> TableInherits<Other> for T
where
    Other: HasPrimaryKey,
    T: HasPrimaryKey,
    T::PrimaryKey: SingletonForeignKey<ReferencedTable = Other>
        + TypedColumn<Type = <Other::PrimaryKey as TypedColumn>::Type>,
{
}
