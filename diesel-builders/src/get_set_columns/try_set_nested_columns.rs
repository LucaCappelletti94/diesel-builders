//! Trait for fallibly setting multiple nested columns.
use crate::{TrySetColumn, TypedColumn, TypedNestedTuple, columns::NestedColumns};
use tuplities::prelude::TuplePopFront;

/// Trait indicating a builder can fallibly set multiple columns.
pub trait TrySetNestedColumns<Error, CS: NestedColumns> {
    /// Attempt to set the values of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if any column cannot be set.
    fn try_set_nested_columns(&mut self, values: CS::NestedTupleType) -> Result<&mut Self, Error>;
}

impl<T, Error> TrySetNestedColumns<Error, ()> for T {
    #[inline]
    fn try_set_nested_columns(&mut self, _values: ()) -> Result<&mut Self, Error> {
        Ok(self)
    }
}

impl<C1, T, Error> TrySetNestedColumns<Error, (C1,)> for T
where
    T: TrySetColumn<C1>,
    C1: TypedColumn,
    Error: From<<T as TrySetColumn<C1>>::Error>,
{
    #[inline]
    fn try_set_nested_columns(&mut self, values: (C1::Type,)) -> Result<&mut Self, Error> {
        self.try_set_column(values.0)?;
        Ok(self)
    }
}

impl<Chead, CTail, T, Error> TrySetNestedColumns<Error, (Chead, CTail)> for T
where
    Chead: crate::TypedColumn,
    CTail: NestedColumns,
    (Chead, CTail): NestedColumns,
    T: TrySetColumn<Chead> + TrySetNestedColumns<Error, CTail>,
    Error: From<<T as TrySetColumn<Chead>>::Error>,
    <(Chead, CTail) as TypedNestedTuple>::NestedTupleType:
        TuplePopFront<Front = Chead::Type, Tail = (CTail::NestedTupleType,)>,
{
    #[inline]
    fn try_set_nested_columns(
        &mut self,
        values: <(Chead, CTail) as TypedNestedTuple>::NestedTupleType,
    ) -> Result<&mut Self, Error> {
        let (head, (tail,)) = values.pop_front();
        self.try_set_column(head)?;
        self.try_set_nested_columns(tail)?;
        Ok(self)
    }
}
