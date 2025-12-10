//! Trait for attempting to set columns in a mandatory same-as relationship.

use crate::{
    MandatorySameAsIndex, TrySetMandatorySameAsColumn, TypedColumn, columns::NestedColumns,
};

/// Trait to try set a column in a mandatory same-as relationship.
pub trait TrySetMandatorySameAsNestedColumns<Type, Error, Keys: NestedColumns, CS: NestedColumns> {
    /// Attempt to set the value of the specified columns in the mandatory
    /// same-as relationship.
    ///
    /// # Errors
    ///
    /// Returns an error if the column values cannot be set in the mandatory
    /// same-as relationship.
    fn try_set_mandatory_same_as_columns(&mut self, value: &Type) -> Result<&mut Self, Error>;
}

impl<T, Type, Error> TrySetMandatorySameAsNestedColumns<Type, Error, (), ()> for T {
    #[inline]
    fn try_set_mandatory_same_as_columns(&mut self, _value: &Type) -> Result<&mut Self, Error> {
        Ok(self)
    }
}

impl<T, Error, Key: MandatorySameAsIndex, Column: TypedColumn<Table = Key::ReferencedTable>>
    TrySetMandatorySameAsNestedColumns<Column::Type, Error, (Key,), (Column,)> for T
where
    T: TrySetMandatorySameAsColumn<Key, Column>,
    Error: From<<T as TrySetMandatorySameAsColumn<Key, Column>>::Error>,
{
    #[inline]
    fn try_set_mandatory_same_as_columns(
        &mut self,
        value: &Column::Type,
    ) -> Result<&mut Self, Error> {
        Ok(self.try_set_mandatory_same_as_column(value.clone())?)
    }
}

impl<
    T,
    Error,
    KeysHead: MandatorySameAsIndex,
    KeysTail: NestedColumns,
    ColumnsHead: TypedColumn<Table = KeysHead::ReferencedTable>,
    ColumnsTail: NestedColumns,
>
    TrySetMandatorySameAsNestedColumns<
        ColumnsHead::Type,
        Error,
        (KeysHead, KeysTail),
        (ColumnsHead, ColumnsTail),
    > for T
where
    (KeysHead, KeysTail): NestedColumns,
    (ColumnsHead, ColumnsTail): NestedColumns,
    T: TrySetMandatorySameAsColumn<KeysHead, ColumnsHead>
        + TrySetMandatorySameAsNestedColumns<ColumnsHead::Type, Error, KeysTail, ColumnsTail>,
    Error: From<<T as TrySetMandatorySameAsColumn<KeysHead, ColumnsHead>>::Error>,
{
    #[inline]
    fn try_set_mandatory_same_as_columns(
        &mut self,
        value: &ColumnsHead::Type,
    ) -> Result<&mut Self, Error> {
        self.try_set_mandatory_same_as_column(value.clone())?;
        self.try_set_mandatory_same_as_columns(value)?;
        Ok(self)
    }
}
