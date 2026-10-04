//! Trait for builders which may get multiple nested columns.
use tuplities::prelude::IntoNestedTupleOption;

use crate::{MayGetColumn, TableExt, TypedColumn, columns::NonEmptyNestedProjection};

/// Trait indicating a builder which may get multiple columns.
pub trait MayGetNestedColumns<CS: NonEmptyNestedProjection> {
    /// May get the owned values of the specified columns.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::MayGetNestedColumns;
    /// use schema::{User, users};
    ///
    /// let mut conn = connection_with_data()?;
    /// let user = users::table.find(1).first::<User>(&mut conn)?;
    /// let present: Option<User> = Some(user);
    /// let (name, (nickname,)) =
    ///     MayGetNestedColumns::<(users::name, (users::nickname,))>::may_get_nested_columns(&present);
    /// assert_eq!(name, Some("Ada".to_string()));
    /// assert_eq!(nickname, Some(Some("Ace".to_string())));
    /// let absent: Option<User> = None;
    /// let (age, (nickname,)) =
    ///     MayGetNestedColumns::<(users::age, (users::nickname,))>::may_get_nested_columns(&absent);
    /// assert_eq!(age, None);
    /// assert_eq!(nickname, None);
    /// # Ok(())
    /// # }
    /// ```
    fn may_get_nested_columns(
        &self,
    ) -> <CS::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions;
}

impl<C1, T> MayGetNestedColumns<(C1,)> for T
where
    T: MayGetColumn<C1>,
    C1: TypedColumn<Table: TableExt>,
{
    #[inline]
    fn may_get_nested_columns(&self) -> (Option<C1::ColumnType>,) {
        (self.may_get_column(),)
    }
}

impl<CHead, CTail, T> MayGetNestedColumns<(CHead, CTail)> for T
where
    CHead: TypedColumn,
    CTail: NonEmptyNestedProjection,
    (CHead, CTail): NonEmptyNestedProjection<
        NestedTupleColumnType = (CHead::ColumnType, CTail::NestedTupleColumnType),
    >,
    T: MayGetColumn<CHead> + MayGetNestedColumns<CTail>,
{
    #[inline]
    fn may_get_nested_columns(
        &self,
    ) -> (
        Option<CHead::ColumnType>,
        <CTail::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
    ) {
        (self.may_get_column(), self.may_get_nested_columns())
    }
}
