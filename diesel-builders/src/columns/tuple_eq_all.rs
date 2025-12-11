//! Submodule providing a nested tuple version of the `EqAll` trait for Diesel columns.

use diesel::{Expression, expression::AsExpression, sql_types::SingleValue};
use tuplities::prelude::FlattenNestedTuple;

use crate::{TypedColumn, TypedNestedTuple};

/// Trait for creating a tuple of equality expressions that compare all elements.
pub trait TupleEqAll: TypedNestedTuple {
    /// The output type of the equality operation.
    type EqAll: FlattenNestedTuple;
    /// Creates a tuple of equality tuple comparing all elements.
    fn eq_all(self, rhs: Self::NestedTupleType) -> Self::EqAll;
}

impl TupleEqAll for () {
    type EqAll = ();
    fn eq_all(self, _rhs: ()) -> Self::EqAll {}
}

impl<Head> TupleEqAll for (Head,)
where
    Head: TypedColumn<Type: AsExpression<<Head as diesel::Expression>::SqlType>>
        + Expression<SqlType: SingleValue>,
{
    type EqAll = (diesel::dsl::Eq<Head, Head::Type>,);
    fn eq_all(self, rhs: (Head::Type,)) -> Self::EqAll {
        use diesel::ExpressionMethods;
        (self.0.eq(rhs.0),)
    }
}

impl<Head, Tail> TupleEqAll for (Head, Tail)
where
    Head: TypedColumn<Type: AsExpression<<Head as diesel::Expression>::SqlType>>
        + Expression<SqlType: SingleValue>,
    Tail: TupleEqAll,
    (Head, Tail): TypedNestedTuple<NestedTupleType = (Head::Type, Tail::NestedTupleType)>,
    (diesel::dsl::Eq<Head, Head::Type>, Tail::EqAll): FlattenNestedTuple,
{
    type EqAll = (diesel::dsl::Eq<Head, Head::Type>, Tail::EqAll);
    fn eq_all(self, rhs: (Head::Type, Tail::NestedTupleType)) -> Self::EqAll {
        use diesel::ExpressionMethods;
        (self.0.eq(rhs.0), self.1.eq_all(rhs.1))
    }
}
