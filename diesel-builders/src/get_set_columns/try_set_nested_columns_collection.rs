//! Trait for fallibly setting multiple columns from a collection.

use crate::{
    TypedNestedTupleCollection, columns::NestedColumns, get_set_columns::TrySetNestedColumns,
};
use tuplities::prelude::TuplePopFront;

/// Trait indicating a builder can fallibly set multiple columns.
pub trait TrySetColumnsCollection<Error, ColumnsCollection: TypedNestedTupleCollection> {
    /// Attempt to set the `nested_values` of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if any column cannot be set.
    fn try_set_nested_columns_collection(
        &mut self,
        nested_values: ColumnsCollection::NestedCollectionType,
    ) -> Result<&mut Self, Error>;
}

impl<T, Error> TrySetColumnsCollection<Error, ()> for T {
    #[inline]
    fn try_set_nested_columns_collection(
        &mut self,
        _nested_values: (),
    ) -> Result<&mut Self, Error> {
        Ok(self)
    }
}

impl<C1, T, Error> TrySetColumnsCollection<Error, (C1,)> for T
where
    T: TrySetNestedColumns<Error, C1>,
    C1: NestedColumns,
{
    #[inline]
    fn try_set_nested_columns_collection(
        &mut self,
        nested_values: <(C1,) as TypedNestedTupleCollection>::NestedCollectionType,
    ) -> Result<&mut Self, Error> {
        self.try_set_nested_columns(nested_values.0)
    }
}

impl<T, Chead, CTail, Error> TrySetColumnsCollection<Error, (Chead, CTail)> for T
where
    Chead: NestedColumns,
    CTail: TypedNestedTupleCollection,
    (Chead, CTail): TypedNestedTupleCollection,
    T: TrySetNestedColumns<Error, Chead> + TrySetColumnsCollection<Error, CTail>,
    <(Chead, CTail) as TypedNestedTupleCollection>::NestedCollectionType: TuplePopFront<
            Front = Chead::NestedTupleType,
            Tail = (<CTail as TypedNestedTupleCollection>::NestedCollectionType,),
        >,
{
    #[inline]
    fn try_set_nested_columns_collection(
        &mut self,
        nested_values: <(Chead, CTail) as TypedNestedTupleCollection>::NestedCollectionType,
    ) -> Result<&mut Self, Error> {
        let (head, (tail,)) = nested_values.pop_front();
        self.try_set_nested_columns(head)?;
        self.try_set_nested_columns_collection(tail)?;
        Ok(self)
    }
}
