//! Submodule defining a trait to iterate the foreign keys in a table
//! which reference the same foreign index in another table.

use tuplities::prelude::{IntoNestedTupleOption, NestedTupleOption, NestedTupleRef};

use crate::{
    TryGetDynamicColumns, TypedNestedTuple,
    builder_error::DynamicColumnError,
    columns::{
        HasNestedDynColumns, NestedDynColumns, NonEmptyNestedProjection, NonEmptyProjection,
    },
    get_column::dynamic_multi::sealed::VariadicTryGetDynamicColumns,
};

mod blankets;

/// Alias for the reference type of a nested tuple value.
type Ref<'a, T> = <<T as TypedNestedTuple>::NestedTupleValueType as NestedTupleRef>::Ref<'a>;
/// Alias for the optional reference type of a nested tuple value.
type OptRef<'a, T> = <Ref<'a, T> as IntoNestedTupleOption>::IntoOptions;

/// An iterator over foreign keys in a table which reference the same foreign
/// dynamic index. The index is represented as a nested tuple of dynamic
/// columns.
pub trait IterDynForeignKeys<DynIdx: NestedDynColumns>: TryGetDynamicColumns {
    /// Returns an iterator over the foreign keys in this table which reference
    /// the given foreign index.
    ///
    /// # Implementation details
    ///
    /// This method, due to its dynamic nature, is able to handle the cases
    /// where a table does not have any foreign keys referencing the given
    /// dynamic index, returning an empty iterator in such cases. This is not
    /// as easily achievable with static typing, where it is needfull to
    /// implement a different trait for each possible foreign unique index,
    /// since at the time of writing the specialization feature is still
    /// unstable in Rust. If you need only to work with statically known
    /// foreign keys, and all tables in your hierarchies have at least one
    /// foreign key referencing the indices on which you want to join,
    /// consider using the [`IterForeignKeys`] trait instead, which provides
    /// better compile-time guarantees and does not require either dynamic
    /// column handling, or error handling.
    ///
    /// # Arguments
    ///
    /// * `index` - The dynamic index to match foreign keys against.
    ///
    /// # Returns
    ///
    /// * An iterator over the foreign keys in this table which reference the
    ///   given foreign index.
    fn iter_foreign_key_dyn_columns(index: DynIdx) -> impl Iterator<Item = DynIdx>;

    /// Returns an iterator over the foreign keys in this table which reference
    /// the given foreign index. Foreign keys with `None` values are included.
    ///
    /// # Arguments
    ///
    /// * `index` - The dynamic index to match foreign keys against.
    ///
    /// # Errors
    ///
    /// As described in the [`IterDynForeignKeys::iter_foreign_key_dyn_columns`]
    /// method, this method is dynamic in nature, and may fail if, due to
    /// antagonistic parameterization of the provided index, the foreign
    /// keys cannot be retrieved.
    fn iter_dyn_match_simple<'a>(
        &'a self,
        index: DynIdx,
    ) -> impl Iterator<Item = Result<OptRef<'a, DynIdx>, DynamicColumnError>>
    where
        DynIdx: 'a + VariadicTryGetDynamicColumns<'a, Self>,
    {
        Self::iter_foreign_key_dyn_columns(index).map(|keys| self.try_get_dynamic_columns_ref(keys))
    }

    /// Returns an iterator over the foreign keys in this table which reference
    /// the given foreign index. Foreign keys with `None` values are skipped.
    ///
    /// # Arguments
    ///
    /// * `index` - The dynamic index to match foreign keys against.
    ///
    /// # Errors
    ///
    /// As described in the [`IterDynForeignKeys::iter_foreign_key_dyn_columns`]
    /// method, this method is dynamic in nature, and may fail if, due to
    /// antagonistic parameterization of the provided index, the foreign
    /// keys cannot be retrieved.
    fn iter_dyn_match_full<'a>(
        &'a self,
        index: DynIdx,
    ) -> impl Iterator<Item = Result<Ref<'a, DynIdx>, DynamicColumnError>>
    where
        DynIdx: 'a + VariadicTryGetDynamicColumns<'a, Self>,
    {
        self.iter_dyn_match_simple(index)
            .filter_map(|res| res.map(NestedTupleOption::transpose).transpose())
    }
}

/// An iterator over foreign keys in a table which reference the same foreign
/// index in another table. The index is represented as a nested tuple of
/// diesel column marker structs.
pub trait IterForeignKeys<NestedIdx: HasNestedDynColumns + NonEmptyNestedProjection> {
    /// Returns an iterator over the foreign keys in this table.
    ///
    /// This method will not be available in table hierarchies if any table in
    /// the hierarchy does not have at least one foreign key referencing the
    /// given foreign index. If you need to handle such cases, consider using
    /// the [`IterDynForeignKeys`] trait instead.
    fn iter_foreign_key_columns()
    -> impl Iterator<Item = <NestedIdx as HasNestedDynColumns>::NestedDynColumns>;

    /// Returns an iterator over the foreign keys in this table which reference
    /// the given foreign index. Foreign keys with `None` values are included.
    ///
    /// This method will not be available in table hierarchies if any table in
    /// the hierarchy does not have at least one foreign key referencing the
    /// given foreign index. If you need to handle such cases, consider using
    /// the [`IterDynForeignKeys`] trait instead.
    fn iter_match_simple<'a>(&'a self) -> impl Iterator<Item = OptRef<'a, NestedIdx>>
    where
        NestedIdx: 'a;

    /// Returns an iterator over the foreign keys in this table which reference
    /// the given foreign index. Foreign keys with `None` values are skipped.
    ///
    /// This method will not be available in table hierarchies if any table in
    /// the hierarchy does not have at least one foreign key referencing the
    /// given foreign index. If you need to handle such cases, consider using
    /// the [`IterDynForeignKeys`] trait instead.
    fn iter_match_full<'a>(&'a self) -> impl Iterator<Item = Ref<'a, NestedIdx>>
    where
        NestedIdx: 'a,
    {
        self.iter_match_simple().filter_map(NestedTupleOption::transpose)
    }
}

/// An extension of the [`IterForeignKeys`] trait moving the generic parameter
/// from the trait to the method to facilitate usage in certain contexts.
pub trait IterForeignKeyExt {
    #[inline]
    /// Returns an iterator over the foreign keys in this table which reference
    /// the given foreign index. Foreign keys with `None` values are included.
    ///
    /// This method will not be available in table hierarchies if any table in
    /// the hierarchy does not have at least one foreign key referencing the
    /// given foreign index. If you need to handle such cases, consider using
    /// the [`IterForeignKeyExt::iter_dynamic_match_simple`] method instead.
    fn iter_match_simple<'a, Idx>(&'a self) -> impl Iterator<Item = OptRef<'a, Idx::Nested>>
    where
        Idx: NonEmptyProjection<Nested: HasNestedDynColumns> + 'a,
        Self: IterForeignKeys<Idx::Nested>,
    {
        IterForeignKeys::iter_match_simple(self)
    }

    #[inline]
    /// Returns an iterator over the DYNAMIC foreign keys in this table which
    /// reference the given foreign index. Foreign keys with `None` values
    /// are included.
    ///
    /// # Implementation details
    ///
    /// This method leverages dynamic column retrieval to provide an iterator
    /// which can handle the case where no foreign keys reference the given
    /// dynamic index, returning an empty iterator in such cases. If you need
    /// only to work with statically known foreign keys, and all tables in your
    /// hierarchies have at least one foreign key referencing the indices on
    /// which you want to join, consider using the
    /// [`IterForeignKeyExt::iter_match_simple`]
    ///
    /// # Errors
    ///
    /// Read the documentation of [`IterDynForeignKeys::iter_dyn_match_simple`]
    /// for details on possible errors.
    fn iter_dynamic_match_simple<'a, DynIdx>(
        &'a self,
        index: DynIdx,
    ) -> impl Iterator<Item = Result<OptRef<'a, DynIdx>, DynamicColumnError>>
    where
        DynIdx: NestedDynColumns + VariadicTryGetDynamicColumns<'a, Self> + 'a,
        Self: IterDynForeignKeys<DynIdx>,
    {
        IterDynForeignKeys::iter_dyn_match_simple(self, index)
    }

    #[inline]
    /// Returns an iterator over the foreign keys in this table which reference
    /// the given foreign index. Foreign keys with `None` values are skipped.
    ///
    /// This method will not be available in table hierarchies if any table in
    /// the hierarchy does not have at least one foreign key referencing the
    /// given foreign index. If you need to handle such cases, consider using
    /// the [`IterForeignKeyExt::iter_dynamic_match_full`] method instead.
    fn iter_match_full<'a, Idx>(&'a self) -> impl Iterator<Item = Ref<'a, Idx::Nested>>
    where
        Idx: NonEmptyProjection<Nested: HasNestedDynColumns> + 'a,
        Self: IterForeignKeys<Idx::Nested>,
    {
        <Self as IterForeignKeys<Idx::Nested>>::iter_match_full(self)
    }

    #[inline]
    /// Returns an iterator over the DYNAMIC foreign keys in this table which
    /// reference the given foreign index. Foreign keys with `None` values
    /// are skipped.
    ///
    /// # Implementation details
    ///
    /// This method leverages dynamic column retrieval to provide an iterator
    /// which can handle the case where no foreign keys reference the given
    /// dynamic index, returning an empty iterator in such cases. If you need
    /// only to work with statically known foreign keys, and all tables in your
    /// hierarchies have at least one foreign key referencing the indices on
    /// which you want to join, consider using the
    /// [`IterForeignKeyExt::iter_match_full`] method instead.
    ///
    /// # Errors
    ///
    /// Read the documentation of [`IterDynForeignKeys::iter_dyn_match_full`]
    /// for details on possible errors.
    fn iter_dynamic_match_full<'a, DynIdx>(
        &'a self,
        index: DynIdx,
    ) -> impl Iterator<Item = Result<Ref<'a, DynIdx>, DynamicColumnError>>
    where
        DynIdx: NestedDynColumns + VariadicTryGetDynamicColumns<'a, Self> + 'a,
        Self: IterDynForeignKeys<DynIdx>,
    {
        <Self as IterDynForeignKeys<DynIdx>>::iter_dyn_match_full(self, index)
    }

    #[must_use]
    #[inline]
    /// Returns an iterator over the foreign keys in this table.
    ///
    /// This method will not be available in table hierarchies if any table in
    /// the hierarchy does not have at least one foreign key referencing the
    /// given foreign index. If you need to handle such cases, consider using
    /// the [`IterForeignKeyExt::iter_dynamic_foreign_key_columns`] method
    /// instead.
    fn iter_foreign_key_columns<Idx>()
    -> impl Iterator<Item = <Idx::Nested as HasNestedDynColumns>::NestedDynColumns>
    where
        Idx: NonEmptyProjection<Nested: HasNestedDynColumns>,
        Self: IterForeignKeys<Idx::Nested>,
    {
        <Self as IterForeignKeys<Idx::Nested>>::iter_foreign_key_columns()
    }

    #[inline]
    /// Returns an iterator over the DYNAMIC foreign keys in this table.
    ///
    /// This method can handle the case where no foreign keys reference the
    /// given foreign index, returning an empty iterator in such cases. If you
    /// need only to work with statically known foreign keys, and all tables in
    /// your hierarchies have at least one foreign key referencing the indices
    /// on which you want to join, consider using the
    /// [`IterForeignKeyExt::iter_foreign_key_columns`] method instead.
    ///
    /// # Errors
    ///
    /// * Read the documentation of
    ///   [`IterDynForeignKeys::iter_foreign_key_dyn_columns`] for details on
    ///   possible errors.
    fn iter_dynamic_foreign_key_columns<DynIdx>(index: DynIdx) -> impl Iterator<Item = DynIdx>
    where
        DynIdx: NestedDynColumns,
        Self: IterDynForeignKeys<DynIdx>,
    {
        <Self as IterDynForeignKeys<DynIdx>>::iter_foreign_key_dyn_columns(index)
    }
}

impl<T> IterForeignKeyExt for T {}
