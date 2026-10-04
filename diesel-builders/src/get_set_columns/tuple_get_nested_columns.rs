//! Submodule providing the `TupleGetNestedColumns` trait.

use crate::{GetColumn, TableExt, TypedColumn, TypedNestedTuple, columns::NestedColumns};

/// Variant of `GetNestedColumns` for n-uples.
pub trait TupleGetNestedColumns<CS: NestedColumns> {
    /// Get the values of the specified columns as an n-uple.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::TupleGetNestedColumns;
    /// use schema::{Profile, User, profiles, users};
    ///
    /// let mut conn = connection_with_data()?;
    /// let profile = profiles::table.find(1).first::<Profile>(&mut conn)?;
    /// let user = users::table.find(1).first::<User>(&mut conn)?;
    /// let (display, (name,)) =
    ///     TupleGetNestedColumns::<(profiles::display_name, (users::name,))>::tuple_get_nested_columns(
    ///         &(profile, (user,)),
    ///     );
    /// assert_eq!(display, "Ada");
    /// assert_eq!(name, "Ada");
    /// # Ok(())
    /// # }
    /// ```
    fn tuple_get_nested_columns(&self) -> <CS as TypedNestedTuple>::NestedTupleColumnType;
}

impl TupleGetNestedColumns<()> for () {
    #[inline]
    fn tuple_get_nested_columns(&self) {}
}

impl<T1, C1> TupleGetNestedColumns<(C1,)> for (T1,)
where
    T1: GetColumn<C1>,
    C1: TypedColumn<Table: TableExt>,
{
    #[inline]
    fn tuple_get_nested_columns(&self) -> <(C1,) as TypedNestedTuple>::NestedTupleColumnType {
        (self.0.get_column(),)
    }
}

impl<CHead, CTail, THead, TTail> TupleGetNestedColumns<(CHead, CTail)> for (THead, TTail)
where
    CHead: TypedColumn,
    CTail: NestedColumns,
    (CHead, CTail):
        NestedColumns<NestedTupleColumnType = (CHead::ColumnType, CTail::NestedTupleColumnType)>,
    THead: GetColumn<CHead>,
    TTail: TupleGetNestedColumns<CTail>,
{
    #[inline]
    fn tuple_get_nested_columns(
        &self,
    ) -> <(CHead, CTail) as TypedNestedTuple>::NestedTupleColumnType {
        (self.0.get_column(), self.1.tuple_get_nested_columns())
    }
}
