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
    fn try_set_mandatory_same_as_columns(
        &mut self,
        value: &(impl Into<Type> + Clone),
    ) -> Result<&mut Self, Error>;
}

impl<T, Type, Error> TrySetMandatorySameAsNestedColumns<Type, Error, (), ()> for T {
    #[inline]
    fn try_set_mandatory_same_as_columns(
        &mut self,
        _value: &(impl Into<Type> + Clone),
    ) -> Result<&mut Self, Error> {
        Ok(self)
    }
}

impl<
    Type: Clone,
    T,
    Error,
    Key: MandatorySameAsIndex,
    Column: TypedColumn<Table = Key::ReferencedTable>,
> TrySetMandatorySameAsNestedColumns<Type, Error, (Key,), (Column,)> for T
where
    Column::ColumnType: From<Type>,
    T: TrySetMandatorySameAsColumn<Key, Column>,
    Error: From<<T as TrySetMandatorySameAsColumn<Key, Column>>::Error>,
{
    #[inline]
    fn try_set_mandatory_same_as_columns(
        &mut self,
        value: &(impl Into<Type> + Clone),
    ) -> Result<&mut Self, Error> {
        let value: Type = value.clone().into();
        Ok(self.try_set_mandatory_same_as_column(value)?)
    }
}

impl<
    T,
    Error,
    Type: Clone,
    KeysHead: MandatorySameAsIndex,
    KeysTail: NestedColumns,
    CHead: TypedColumn<Table = KeysHead::ReferencedTable>,
    CTail: NestedColumns,
> TrySetMandatorySameAsNestedColumns<Type, Error, (KeysHead, KeysTail), (CHead, CTail)> for T
where
    (KeysHead, KeysTail): NestedColumns,
    (CHead, CTail): NestedColumns,
    CHead::ColumnType: From<Type>,
    T: TrySetMandatorySameAsColumn<KeysHead, CHead>
        + TrySetMandatorySameAsNestedColumns<Type, Error, KeysTail, CTail>,
    Error: From<<T as TrySetMandatorySameAsColumn<KeysHead, CHead>>::Error>,
{
    #[inline]
    fn try_set_mandatory_same_as_columns(
        &mut self,
        value: &(impl Into<Type> + Clone),
    ) -> Result<&mut Self, Error> {
        self.try_set_mandatory_same_as_columns(value)?;
        let value: Type = value.clone().into();
        self.try_set_mandatory_same_as_column(value)?;
        Ok(self)
    }
}
