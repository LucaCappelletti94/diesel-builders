//! Submodule providing a nested tuple version of the `ToOrder` trait for Diesel
//! columns.

use diesel::{Expression, expression::AsExpression, sql_types::SingleValue};
use tuplities::prelude::FlattenNestedTuple;

use crate::{TypedColumn, TypedNestedTuple};

/// Trait for creating a tuple of order expressions.
pub trait TupleToOrder: TypedNestedTuple {
    /// The output type of the order operation.
    type Order: FlattenNestedTuple + Expression;
    /// Creates a tuple of order expressions.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::columns::TupleToOrder;
    /// use schema::users;
    ///
    /// let mut conn = connection_with_data()?;
    /// let order = (users::age, (users::name,)).to_order();
    /// let rows = users::table
    ///     .order(order)
    ///     .select((users::name, users::age))
    ///     .load::<(String, i32)>(&mut conn)?;
    /// assert_eq!(rows, [("Ada".into(), 20), ("Grace".into(), 30)]);
    /// # Ok(())
    /// # }
    /// ```
    fn to_order(self) -> Self::Order;
}

impl<Head> TupleToOrder for (Head,)
where
    Head: TypedColumn<ColumnType: AsExpression<<Head as diesel::Expression>::SqlType>>
        + Expression<SqlType: SingleValue>,
{
    type Order = (Head,);
    fn to_order(self) -> Self::Order {
        (self.0,)
    }
}

impl<Head, Tail> TupleToOrder for (Head, Tail)
where
    Head: TypedColumn<ColumnType: AsExpression<<Head as diesel::Expression>::SqlType>>
        + Expression<SqlType: SingleValue>,
    Tail: TupleToOrder,
    (Head, Tail):
        TypedNestedTuple<NestedTupleColumnType = (Head::ColumnType, Tail::NestedTupleColumnType)>,
    (Head, Tail::Order): FlattenNestedTuple + Expression,
{
    type Order = (Head, Tail::Order);
    fn to_order(self) -> Self::Order {
        (self.0, self.1.to_order())
    }
}
