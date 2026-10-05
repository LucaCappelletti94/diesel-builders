//! Submodule providing the `GetNestedColumns` trait.

use tuplities::prelude::NestedTupleRef;

use crate::{GetColumn, TableExt, TypedColumn, columns::NonEmptyNestedProjection};

/// Trait indicating a builder can get multiple columns.
pub trait GetNestedColumns<CS: NonEmptyNestedProjection> {
    /// Get the values of the specified columns.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::GetNestedColumns;
    /// use schema::{User, users};
    ///
    /// let mut conn = connection_with_data()?;
    /// let user = users::table.find(1).first::<User>(&mut conn)?;
    /// let (name, (age, (nickname,))) = GetNestedColumns::<(
    ///     users::name,
    ///     (users::age, (users::nickname,)),
    /// )>::get_nested_columns(&user);
    /// assert_eq!(name, "Ada");
    /// assert_eq!(age, 20);
    /// assert_eq!(nickname, Some("Ace".to_string()));
    /// # Ok(())
    /// # }
    /// ```
    fn get_nested_columns(&self) -> CS::NestedTupleColumnType;
    /// Get the references of the specified columns.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::GetNestedColumns;
    /// use schema::{User, users};
    ///
    /// let mut conn = connection_with_data()?;
    /// let user = users::table.find(1).first::<User>(&mut conn)?;
    /// let (name, (age,)) =
    ///     GetNestedColumns::<(users::name, (users::age,))>::get_nested_column_refs(&user);
    /// assert_eq!(*name, "Ada");
    /// assert_eq!(*age, 20);
    /// # Ok(())
    /// # }
    /// ```
    fn get_nested_column_refs(&self) -> <CS::NestedTupleColumnType as NestedTupleRef>::Ref<'_>;
}

impl<C1, T> GetNestedColumns<(C1,)> for T
where
    T: GetColumn<C1>,
    C1: TypedColumn<Table: TableExt>,
{
    #[inline]
    fn get_nested_columns(&self) -> (C1::ColumnType,) {
        (self.get_column(),)
    }
    #[inline]
    fn get_nested_column_refs(&self) -> (&C1::ColumnType,) {
        (self.get_column_ref(),)
    }
}

impl<CHead, CTail, T> GetNestedColumns<(CHead, CTail)> for T
where
    CHead: TypedColumn,
    CTail: NonEmptyNestedProjection,
    (CHead, CTail): NonEmptyNestedProjection<
        NestedTupleColumnType = (CHead::ColumnType, CTail::NestedTupleColumnType),
    >,
    T: GetColumn<CHead> + GetNestedColumns<CTail>,
{
    #[inline]
    fn get_nested_columns(&self) -> (CHead::ColumnType, CTail::NestedTupleColumnType) {
        (self.get_column(), self.get_nested_columns())
    }

    #[inline]
    fn get_nested_column_refs(
        &self,
    ) -> (&CHead::ColumnType, <CTail::NestedTupleColumnType as NestedTupleRef>::Ref<'_>) {
        (self.get_column_ref(), self.get_nested_column_refs())
    }
}
