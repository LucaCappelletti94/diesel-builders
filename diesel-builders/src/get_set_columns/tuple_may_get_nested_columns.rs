//! Variant of `MayGetNestedColumns` for n-uples.

use tuplities::prelude::IntoNestedTupleOption;

use crate::{MayGetColumn, TableExt, TypedColumn, columns::NestedColumns};

/// Variant of `MayGetNestedColumns` for n-uples.
pub trait TupleMayGetNestedColumns<CS: NestedColumns> {
    /// May get the values of the specified columns as an n-uple.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::TupleMayGetNestedColumns;
    /// use schema::{Profile, User, profiles, users};
    ///
    /// let mut conn = connection_with_data()?;
    /// let user = users::table.find(2).first::<User>(&mut conn)?;
    /// let absent_profile: Option<Profile> = None;
    /// let present_user: Option<User> = Some(user);
    /// let (visits, (age,)) =
    ///     TupleMayGetNestedColumns::<(profiles::visits, (users::age,))>::tuple_may_get_nested_columns(
    ///         &(absent_profile, (present_user,)),
    ///     );
    /// assert_eq!(visits, None);
    /// assert_eq!(age, Some(30));
    /// # Ok(())
    /// # }
    /// ```
    fn tuple_may_get_nested_columns(
        &self,
    ) -> <CS::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions;
}

impl TupleMayGetNestedColumns<()> for () {
    #[inline]
    fn tuple_may_get_nested_columns(&self) {}
}

impl<T1, C1> TupleMayGetNestedColumns<(C1,)> for (T1,)
where
    T1: MayGetColumn<C1>,
    C1: TypedColumn<Table: TableExt>,
{
    #[inline]
    fn tuple_may_get_nested_columns(&self) -> (Option<C1::ColumnType>,) {
        (self.0.may_get_column(),)
    }
}

impl<CHead, CTail, THead, TTail> TupleMayGetNestedColumns<(CHead, CTail)> for (THead, TTail)
where
    CHead: TypedColumn,
    CTail: NestedColumns,
    (CHead, CTail):
        NestedColumns<NestedTupleColumnType = (CHead::ColumnType, CTail::NestedTupleColumnType)>,
    THead: MayGetColumn<CHead>,
    TTail: TupleMayGetNestedColumns<CTail>,
{
    #[inline]
    fn tuple_may_get_nested_columns(
        &self,
    ) -> (
        Option<CHead::ColumnType>,
        <CTail::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
    ) {
        (self.0.may_get_column(), self.1.tuple_may_get_nested_columns())
    }
}
