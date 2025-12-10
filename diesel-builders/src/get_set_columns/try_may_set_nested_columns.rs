//! Submodule providing the `TryMaySetColumns` trait.

use tuplities::prelude::{IntoNestedTupleOption, TuplePopFront};

use crate::{TrySetColumn, TypedColumn, TypedNestedTuple, columns::NestedColumns};

/// Trait indicating a builder which may try to set multiple columns.
pub trait TryMaySetNestedColumns<Error, CS: NestedColumns> {
    /// Attempt to set the nested_values of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if any column cannot be set.
    fn try_may_set_nested_columns(
        &mut self,
        nested_values: <CS::NestedTupleType as IntoNestedTupleOption>::IntoOptions,
    ) -> Result<&mut Self, Error>;
}

impl<T, Error> TryMaySetNestedColumns<Error, ()> for T {
    #[inline]
    fn try_may_set_nested_columns(&mut self, _nested_values: ()) -> Result<&mut Self, Error> {
        Ok(self)
    }
}

impl<C1, T, Error> TryMaySetNestedColumns<Error, (C1,)> for T
where
    T: crate::TrySetColumn<C1>,
    C1: crate::TypedColumn,
    Error: From<<T as crate::TrySetColumn<C1>>::Error>,
{
    #[inline]
    fn try_may_set_nested_columns(
        &mut self,
        nested_values: <<(C1,) as TypedNestedTuple>::NestedTupleType as IntoNestedTupleOption>::IntoOptions,
    ) -> Result<&mut Self, Error> {
        if let Some(value) = nested_values.0 {
            self.try_set_column(value)?;
        }
        Ok(self)
    }
}

impl<Chead, CTail, T, Error> TryMaySetNestedColumns<Error, (Chead, CTail)> for T
where
    Chead: TypedColumn,
    CTail: NestedColumns,
    (Chead, CTail): NestedColumns,
    T: TrySetColumn<Chead>,
    Error: From<<T as TrySetColumn<Chead>>::Error>,
    T: TryMaySetNestedColumns<Error, CTail>,
    <<(Chead, CTail) as TypedNestedTuple>::NestedTupleType as IntoNestedTupleOption>::IntoOptions:
        TuplePopFront<
                Front = Option<Chead::Type>,
                Tail = (<CTail::NestedTupleType as IntoNestedTupleOption>::IntoOptions,),
            >,
{
    #[inline]
    fn try_may_set_nested_columns(
        &mut self,
        nested_values: <<(Chead, CTail) as TypedNestedTuple>::NestedTupleType as IntoNestedTupleOption>::IntoOptions,
    ) -> Result<&mut Self, Error> {
        let (head, (tail,)) = nested_values.pop_front();
        if let Some(value) = head {
            self.try_set_column(value)?;
        }
        self.try_may_set_nested_columns(tail)?;
        Ok(self)
    }
}
