//! Submodule providing a nested tuple version of the `EqAll` trait for Diesel
//! columns.

use diesel::{Expression, expression::AsExpression, sql_types::SingleValue};
use tuplities::prelude::FlattenNestedTuple;

use crate::{TypedColumn, TypedNestedTuple};

/// Trait for creating a tuple of equality expressions that compare all
/// elements.
pub trait TupleEqAll: TypedNestedTuple {
    /// The output type of the equality operation.
    type EqAll: FlattenNestedTuple;
    /// Creates equality expressions for every column in the nested tuple.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::columns::TupleEqAll;
    /// use schema::users;
    ///
    /// let mut conn = connection_with_data()?;
    /// let (name, (age,)) = TupleEqAll::eq_all((users::name, (users::age,)), ("Ada".into(), (20,)));
    /// let ids = users::table.filter(name).filter(age).select(users::id).load::<i32>(&mut conn)?;
    /// assert_eq!(ids, [1]);
    /// let missing = users::table
    ///     .filter(users::name.eq("Ada"))
    ///     .filter(users::age.eq(30))
    ///     .select(users::id)
    ///     .load::<i32>(&mut conn)?;
    /// assert!(missing.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    fn eq_all(self, rhs: Self::NestedTupleColumnType) -> Self::EqAll;
}

impl<Head> TupleEqAll for (Head,)
where
    Head: TypedColumn<ColumnType: AsExpression<<Head as diesel::Expression>::SqlType>>
        + Expression<SqlType: SingleValue>,
{
    type EqAll = (diesel::dsl::Eq<Head, Head::ColumnType>,);
    fn eq_all(self, rhs: (Head::ColumnType,)) -> Self::EqAll {
        use diesel::ExpressionMethods;
        (self.0.eq(rhs.0),)
    }
}

impl<Head, Tail> TupleEqAll for (Head, Tail)
where
    Head: TypedColumn<ColumnType: AsExpression<<Head as diesel::Expression>::SqlType>>
        + Expression<SqlType: SingleValue>,
    Tail: TupleEqAll,
    (Head, Tail):
        TypedNestedTuple<NestedTupleColumnType = (Head::ColumnType, Tail::NestedTupleColumnType)>,
    (diesel::dsl::Eq<Head, Head::ColumnType>, Tail::EqAll): FlattenNestedTuple,
{
    type EqAll = (diesel::dsl::Eq<Head, Head::ColumnType>, Tail::EqAll);
    fn eq_all(self, rhs: (Head::ColumnType, Tail::NestedTupleColumnType)) -> Self::EqAll {
        use diesel::ExpressionMethods;
        (self.0.eq(rhs.0), self.1.eq_all(rhs.1))
    }
}
