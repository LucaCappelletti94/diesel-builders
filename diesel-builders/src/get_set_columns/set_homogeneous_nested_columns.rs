//! Trait indicating a builder can set multiple columns.

use crate::{
    SetColumn, TypedColumn,
    columns::{HomogeneouslyTypedNestedColumns, NonEmptyNestedProjection},
};

/// Trait indicating a builder can set multiple columns.
pub trait SetHomogeneousNestedColumns<Type, CS: HomogeneouslyTypedNestedColumns<Type>> {
    /// Set the `nested_values` of the specified columns.
    fn set_homogeneous_nested_columns(&mut self, value: Type) -> &mut Self;
}

impl<Type, T> SetHomogeneousNestedColumns<Type, ()> for T {
    #[inline]
    fn set_homogeneous_nested_columns(&mut self, _value: Type) -> &mut Self {
        self
    }
}

impl<C1, T> SetHomogeneousNestedColumns<C1::Type, (C1,)> for T
where
    T: SetColumn<C1>,
    C1: TypedColumn,
{
    #[inline]
    fn set_homogeneous_nested_columns(&mut self, value: C1::Type) -> &mut Self {
        self.set_column(value)
    }
}

impl<Chead, CTail, T> SetHomogeneousNestedColumns<Chead::Type, (Chead, CTail)> for T
where
    Chead: TypedColumn,
    CTail: HomogeneouslyTypedNestedColumns<Chead::Type>,
    (Chead, CTail):
        NonEmptyNestedProjection<NestedTupleType = (Chead::Type, CTail::NestedTupleType)>,
    T: SetColumn<Chead> + SetHomogeneousNestedColumns<Chead::Type, CTail>,
{
    #[inline]
    fn set_homogeneous_nested_columns(&mut self, value: Chead::Type) -> &mut Self {
        self.set_column(value.clone());
        self.set_homogeneous_nested_columns(value);
        self
    }
}
