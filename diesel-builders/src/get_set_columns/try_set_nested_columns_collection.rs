//! Trait for fallibly setting multiple columns from a collection.

use crate::{
    TypedNestedTupleCollection, columns::NonEmptyNestedProjection,
    get_set_columns::TrySetNestedColumns,
};

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

impl<C1, T, Error> TrySetColumnsCollection<Error, (C1,)> for T
where
    T: TrySetNestedColumns<Error, C1>,
    C1: NonEmptyNestedProjection,
{
    #[inline]
    fn try_set_nested_columns_collection(
        &mut self,
        nested_values: (C1::NestedTupleColumnType,),
    ) -> Result<&mut Self, Error> {
        self.try_set_nested_columns(nested_values.0)
    }
}

impl<T, CHead, CTail, Error> TrySetColumnsCollection<Error, (CHead, CTail)> for T
where
    CHead: NonEmptyNestedProjection,
    CTail: TypedNestedTupleCollection,
    (CHead, CTail): TypedNestedTupleCollection<
        NestedCollectionType = (
            CHead::NestedTupleColumnType,
            <CTail as TypedNestedTupleCollection>::NestedCollectionType,
        ),
    >,
    T: TrySetNestedColumns<Error, CHead> + TrySetColumnsCollection<Error, CTail>,
{
    #[inline]
    fn try_set_nested_columns_collection(
        &mut self,
        (head, tail): (
            CHead::NestedTupleColumnType,
            <CTail as TypedNestedTupleCollection>::NestedCollectionType,
        ),
    ) -> Result<&mut Self, Error> {
        self.try_set_nested_columns(head)?;
        self.try_set_nested_columns_collection(tail)?;
        Ok(self)
    }
}
