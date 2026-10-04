//! Trait for fallibly setting multiple columns from a collection.

use crate::{
    TypedNestedTupleCollection,
    columns::NonEmptyNestedProjection,
    mutation::{MutationContext, PrepareColumns, PrepareColumnsCollection},
};

/// Trait indicating a builder can fallibly set multiple columns.
pub trait TrySetColumnsCollection<Error, ColumnsCollection: TypedNestedTupleCollection> {
    /// Attempt to set the `nested_values` of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if any column cannot be set.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::{MayGetColumn, TrySetColumnsCollection};
    /// use schema::{ValidationError, users};
    ///
    /// let mut values = user_values("Ada", 18, None);
    /// type Groups = ((users::name, (users::age,)), ((users::nickname,),));
    /// TrySetColumnsCollection::<ValidationError, Groups>::try_set_nested_columns_collection(
    ///     &mut values,
    ///     (("Grace".to_string(), (20,)), ((Some("Ace".to_string()),),)),
    /// )?;
    /// assert_eq!(MayGetColumn::<users::name>::may_get_column(&values), Some("Grace".to_string()));
    /// assert_eq!(MayGetColumn::<users::age>::may_get_column(&values), Some(20));
    /// assert_eq!(
    ///     MayGetColumn::<users::nickname>::may_get_column(&values),
    ///     Some(Some("Ace".to_string()))
    /// );
    /// assert_eq!(
    ///     TrySetColumnsCollection::<ValidationError, Groups>::try_set_nested_columns_collection(
    ///         &mut values,
    ///         (("Ada".to_string(), (17,)), ((None,),)),
    ///     )
    ///     .err(),
    ///     Some(ValidationError::AgeTooYoung)
    /// );
    /// assert_eq!(MayGetColumn::<users::name>::may_get_column(&values), Some("Grace".to_string()));
    /// assert_eq!(MayGetColumn::<users::age>::may_get_column(&values), Some(20));
    /// # Ok(())
    /// # }
    /// ```
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
    T: PrepareColumns<Error, C1>,
    C1: NonEmptyNestedProjection,
{
    #[inline]
    fn try_set_nested_columns_collection(
        &mut self,
        nested_values: (C1::NestedTupleColumnType,),
    ) -> Result<&mut Self, Error> {
        let context = MutationContext::default();
        let prepared = <T as PrepareColumnsCollection<Error, (C1,)>>::prepare_columns_collection(
            self,
            nested_values,
            &context,
        )
        .map_err(|(_, error)| error)?;
        <T as PrepareColumnsCollection<Error, (C1,)>>::apply_columns_collection(self, prepared);
        Ok(self)
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
    T: PrepareColumns<Error, CHead> + PrepareColumnsCollection<Error, CTail>,
{
    #[inline]
    fn try_set_nested_columns_collection(
        &mut self,
        (head, tail): (
            CHead::NestedTupleColumnType,
            <CTail as TypedNestedTupleCollection>::NestedCollectionType,
        ),
    ) -> Result<&mut Self, Error> {
        let context = MutationContext::default();
        let prepared =
            <T as PrepareColumnsCollection<Error, (CHead, CTail)>>::prepare_columns_collection(
                self,
                (head, tail),
                &context,
            )
            .map_err(|(_, error)| error)?;
        <T as PrepareColumnsCollection<Error, (CHead, CTail)>>::apply_columns_collection(
            self, prepared,
        );
        Ok(self)
    }
}
