//! Submodule providing the `TrySetHomogeneousNestedColumnsCollection` trait.

use tuplities::prelude::NestedTupleReplicate;

use crate::{
    TypedNestedTupleCollection, columns::NestedColumnsCollection,
    get_set_columns::TrySetColumnsCollection,
};

/// Trait indicating a builder can try to set multiple homogeneous columns.
pub trait TrySetHomogeneousNestedColumnsCollection<Error, Type, NCC: NestedColumnsCollection>:
    TrySetColumnsCollection<Error, NCC>
{
    /// Assigns the same value group to each selected column group.
    ///
    /// # Errors
    ///
    /// Returns an error if any column cannot be set.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::{MayGetColumn, TrySetHomogeneousNestedColumnsCollection};
    /// use schema::{ValidationError, checked_children};
    ///
    /// let mut values = checked_children::table::empty_new_values();
    /// type Groups = ((checked_children::mandatory_id,), ((checked_children::discretionary_id,),));
    /// TrySetHomogeneousNestedColumnsCollection::<ValidationError, (i32,), Groups>::try_set_homogeneous_nested_columns_collection(
    ///     &mut values,
    ///     (1,),
    /// )?;
    /// assert_eq!(
    ///     MayGetColumn::<checked_children::mandatory_id>::may_get_column(&values),
    ///     Some(1)
    /// );
    /// assert_eq!(
    ///     MayGetColumn::<checked_children::discretionary_id>::may_get_column(&values),
    ///     Some(1)
    /// );
    /// # Ok(())
    /// # }
    /// ```
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
