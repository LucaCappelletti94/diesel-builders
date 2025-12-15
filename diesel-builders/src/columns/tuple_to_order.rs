//! Submodule providing a nested tuple version of the `ToOrder` trait for Diesel columns.

use diesel::{Expression, expression::AsExpression, sql_types::SingleValue};
use tuplities::prelude::FlattenNestedTuple;

use crate::{TypedColumn, TypedNestedTuple};

/// Trait for creating a tuple of order expressions.
pub trait TupleToOrder: TypedNestedTuple {
    /// The output type of the order operation.
    type Order: FlattenNestedTuple + Expression;
    /// Creates a tuple of order expressions.
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
    (Head, Tail): TypedNestedTuple<NestedTupleType = (Head::ColumnType, Tail::NestedTupleType)>,
    (Head, Tail::Order): FlattenNestedTuple + Expression,
{
    type Order = (Head, Tail::Order);
    fn to_order(self) -> Self::Order {
        (self.0, self.1.to_order())
    }
}
