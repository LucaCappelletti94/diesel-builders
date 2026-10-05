//! Trait indicating a builder which may set multiple columns.

use tuplities::prelude::IntoNestedTupleOption;

use crate::{MaySetColumn, TableExt, TypedColumn, columns::NonEmptyNestedProjection};

/// Trait indicating a builder which may set multiple columns.
pub trait MaySetColumns<CS: NonEmptyNestedProjection> {
    /// May set the `nested_values` of the specified columns.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::{MayGetColumn, MaySetColumns};
    /// use schema::users;
    ///
    /// let mut values = user_values("Ada", 20, None);
    /// let nickname = (Some(Some("Ace".to_string())),);
    /// MaySetColumns::<(users::nickname,)>::may_set_nested_columns(&mut values, nickname);
    /// assert_eq!(
    ///     MayGetColumn::<users::nickname>::may_get_column(&values),
    ///     Some(Some("Ace".to_string()))
    /// );
    /// MaySetColumns::<(users::nickname,)>::may_set_nested_columns(&mut values, (None,));
    /// assert_eq!(
    ///     MayGetColumn::<users::nickname>::may_get_column(&values),
    ///     Some(Some("Ace".to_string()))
    /// );
    ///
    /// // A two-column group recurses through the head and the tail.
    /// MaySetColumns::<(users::name, (users::nickname,))>::may_set_nested_columns(
    ///     &mut values,
    ///     (Some("Grace".to_string()), (Some(None),)),
    /// );
    /// assert_eq!(MayGetColumn::<users::name>::may_get_column(&values), Some("Grace".to_string()));
    /// assert_eq!(MayGetColumn::<users::nickname>::may_get_column(&values), Some(None));
    /// # }
    /// ```
    fn may_set_nested_columns(
        &mut self,
        nested_values: <CS::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
    ) -> &mut Self;
}

impl<C1, T> MaySetColumns<(C1,)> for T
where
    T: MaySetColumn<C1>,
    C1: TypedColumn<Table: TableExt>,
{
    #[inline]
    fn may_set_nested_columns(&mut self, (value,): (Option<C1::ColumnType>,)) -> &mut Self {
        self.may_set_column(value);
        self
    }
}

impl<CHead, CTail, T> MaySetColumns<(CHead, CTail)> for T
where
    CHead: TypedColumn,
    CTail: NonEmptyNestedProjection,
    (CHead, CTail): NonEmptyNestedProjection<
        NestedTupleColumnType = (CHead::ColumnType, CTail::NestedTupleColumnType),
    >,
    T: MaySetColumn<CHead> + MaySetColumns<CTail>,
{
    #[inline]
    fn may_set_nested_columns(
        &mut self,
        (head, tail): (
            Option<CHead::ColumnType>,
            <CTail::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
        ),
    ) -> &mut Self {
        self.may_set_column(head);
        self.may_set_nested_columns(tail);
        self
    }
}
