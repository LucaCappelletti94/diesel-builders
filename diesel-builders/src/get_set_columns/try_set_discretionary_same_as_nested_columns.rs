//! Trait for attempting to set columns in a discretionary same-as relationship.

use std::convert::Infallible;

use crate::{
    DiscretionarySameAsIndex, OptionalRef, TrySetDiscretionarySameAsColumn, TypedColumn,
    columns::NestedColumns,
};

/// Trait for attempting to set columns in a discretionary same-as relationship.
pub trait TrySetDiscretionarySameAsNestedColumns<
    Type,
    Error,
    Keys: NestedColumns,
    CS: NestedColumns,
>
{
    /// Attempt to set the value of the specified columns in the discretionary
    /// same-as relationship.
    ///
    /// # Errors
    ///
    /// Returns an error if the column values cannot be set in the discretionary
    /// same-as relationship.
    fn try_set_discretionary_same_as_nested_columns(
        &mut self,
        value: &impl OptionalRef<Type>,
    ) -> Result<&mut Self, Error>;
}

impl<T, Type, Error> TrySetDiscretionarySameAsNestedColumns<Type, Error, (), ()> for T {
    #[inline]
    fn try_set_discretionary_same_as_nested_columns(
        &mut self,
        _value: &impl OptionalRef<Type>,
    ) -> Result<&mut Self, Error> {
        Ok(self)
    }
}

impl<
    Type: Clone,
    T,
    Error,
    Key: DiscretionarySameAsIndex,
    Column: TypedColumn<Table = Key::ReferencedTable>,
> TrySetDiscretionarySameAsNestedColumns<Type, Error, (Key,), (Column,)> for T
where
    T: TrySetDiscretionarySameAsColumn<Key, Column>,
    Column::ColumnType: From<Type>,
    Error: From<<T as TrySetDiscretionarySameAsColumn<Key, Column>>::Error>,
{
    #[inline]
    fn try_set_discretionary_same_as_nested_columns(
        &mut self,
        value: &impl OptionalRef<Type>,
    ) -> Result<&mut Self, Error> {
        if let Some(value) = value.as_optional_ref() {
            self.try_set_discretionary_same_as_column(value.clone())?;
        }
        Ok(self)
    }
}

impl<
    T,
    Error,
    Type: Clone,
    KeysHead: DiscretionarySameAsIndex,
    KeysTail: NestedColumns,
    CHead: TypedColumn<Table = KeysHead::ReferencedTable>,
    CTail: NestedColumns,
> TrySetDiscretionarySameAsNestedColumns<Type, Error, (KeysHead, KeysTail), (CHead, CTail)> for T
where
    (KeysHead, KeysTail): NestedColumns,
    (CHead, CTail): NestedColumns,
    CHead::ColumnType: From<Type>,
    T: TrySetDiscretionarySameAsColumn<KeysHead, CHead>
        + TrySetDiscretionarySameAsNestedColumns<Type, Error, KeysTail, CTail>,
    Error: From<<T as TrySetDiscretionarySameAsColumn<KeysHead, CHead>>::Error>,
{
    #[inline]
    fn try_set_discretionary_same_as_nested_columns(
        &mut self,
        value: &impl OptionalRef<Type>,
    ) -> Result<&mut Self, Error> {
        if let Some(value) = value.as_optional_ref() {
            self.try_set_discretionary_same_as_column(value.clone())?;
        }
        <T as TrySetDiscretionarySameAsNestedColumns<
            Type,
            Error,
            KeysTail,
            CTail,
        >>::try_set_discretionary_same_as_nested_columns(self, value)
    }
}

/// Trait for setting columns in a discretionary same-as relationship.
pub trait SetDiscretionarySameAsNestedColumns<Type, Keys: NestedColumns, CS: NestedColumns> {
    /// Attempt to set the value of the specified columns in the discretionary
    /// same-as relationship.
    ///
    /// # Errors
    ///
    /// Returns an error if the column values cannot be set in the discretionary
    /// same-as relationship.
    fn set_discretionary_same_as_nested_columns(
        &mut self,
        value: &impl OptionalRef<Type>,
    ) -> &mut Self;
}

impl<Type, Keys, CS, T> SetDiscretionarySameAsNestedColumns<Type, Keys, CS> for T
where
    Keys: NestedColumns,
    CS: NestedColumns,
    T: TrySetDiscretionarySameAsNestedColumns<Type, Infallible, Keys, CS>,
{
    #[inline]
    fn set_discretionary_same_as_nested_columns(
        &mut self,
        value: &impl OptionalRef<Type>,
    ) -> &mut Self {
        self.try_set_discretionary_same_as_nested_columns(value).unwrap_or_else(|err| match err {})
    }
}
