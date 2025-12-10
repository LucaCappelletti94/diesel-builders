//! Submodule providing the `TrySetHomogeneousNestedColumnsCollection` trait.

use crate::{
    TypedNestedTupleCollection, columns::NestedColumnsCollection,
    get_set_columns::TrySetColumnsCollection,
};
use tuplities::prelude::NestedTupleReplicate;

/// Trait indicating a builder can try to set multiple homogeneous
/// columns.
pub trait TrySetHomogeneousNestedColumnsCollection<Error, Type, NCC: NestedColumnsCollection>:
    TrySetColumnsCollection<Error, NCC>
{
    /// Attempt to set the values of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if any column cannot be set.
    fn try_set_homogeneous_nested_columns_collection(
        &mut self,
        value: Type,
    ) -> Result<&mut Self, Error>;
}

impl<
    Error,
    T,
    Type: Clone,
    NCC: NestedColumnsCollection<NestedCollectionType: NestedTupleReplicate<Type>>,
> TrySetHomogeneousNestedColumnsCollection<Error, Type, NCC> for T
where
    T: TrySetColumnsCollection<Error, NCC>,
{
    #[inline]
    fn try_set_homogeneous_nested_columns_collection(
        &mut self,
        value: Type,
    ) -> Result<&mut Self, Error> {
        let replicates =
            <<NCC as TypedNestedTupleCollection>::NestedCollectionType as NestedTupleReplicate<
                Type,
            >>::nested_tuple_replicate(value);
        <T as TrySetColumnsCollection<Error, NCC>>::try_set_nested_columns_collection(
            self, replicates,
        )
    }
}
