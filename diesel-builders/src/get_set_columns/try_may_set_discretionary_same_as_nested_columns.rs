//! Trait for attempting to set columns in a discretionary same-as relationship.

use crate::{
    DiscretionarySameAsIndex, TryMaySetDiscretionarySameAsColumn, TypedColumn,
    columns::NestedColumns,
};

/// Trait for attempting to set columns in a discretionary same-as relationship.
pub trait TryMaySetDiscretionarySameAsNestedColumns<
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
    fn try_may_set_discretionary_same_as_nested_columns(
        &mut self,
        nested_value: &Type,
    ) -> Result<&mut Self, Error>;
}

impl<T, Type, Error> TryMaySetDiscretionarySameAsNestedColumns<Type, Error, (), ()> for T {
    #[inline]
    fn try_may_set_discretionary_same_as_nested_columns(
        &mut self,
        _nested_value: &Type,
    ) -> Result<&mut Self, Error> {
        Ok(self)
    }
}

impl<T, Error, Key: DiscretionarySameAsIndex, Column: TypedColumn<Table = Key::ReferencedTable>>
    TryMaySetDiscretionarySameAsNestedColumns<Column::ColumnType, Error, (Key,), (Column,)> for T
where
    T: TryMaySetDiscretionarySameAsColumn<Key, Column>,
    Error: From<<T as TryMaySetDiscretionarySameAsColumn<Key, Column>>::Error>,
{
    #[inline]
    fn try_may_set_discretionary_same_as_nested_columns(
        &mut self,
        nested_value: &Column::ColumnType,
    ) -> Result<&mut Self, Error> {
        self.try_may_set_discretionary_same_as_column(nested_value.clone())?;
        Ok(self)
    }
}

impl<
    T,
    Error,
    KeysHead: DiscretionarySameAsIndex,
    KeysTail: NestedColumns,
    ColumnsHead: TypedColumn<Table = KeysHead::ReferencedTable>,
    ColumnsTail: NestedColumns,
>
    TryMaySetDiscretionarySameAsNestedColumns<
        ColumnsHead::ColumnType,
        Error,
        (KeysHead, KeysTail),
        (ColumnsHead, ColumnsTail),
    > for T
where
    (KeysHead, KeysTail): NestedColumns,
    (ColumnsHead, ColumnsTail): NestedColumns,
    T: TryMaySetDiscretionarySameAsColumn<KeysHead, ColumnsHead>
        + TryMaySetDiscretionarySameAsNestedColumns<
            ColumnsHead::ColumnType,
            Error,
            KeysTail,
            ColumnsTail,
        >,
    Error: From<<T as TryMaySetDiscretionarySameAsColumn<KeysHead, ColumnsHead>>::Error>,
{
    #[inline]
    fn try_may_set_discretionary_same_as_nested_columns(
        &mut self,
        nested_value: &ColumnsHead::ColumnType,
    ) -> Result<&mut Self, Error> {
        self.try_may_set_discretionary_same_as_column(nested_value.clone())?;
        <T as TryMaySetDiscretionarySameAsNestedColumns<
            ColumnsHead::ColumnType,
            Error,
            KeysTail,
            ColumnsTail,
        >>::try_may_set_discretionary_same_as_nested_columns(self, nested_value)
    }
}
