//! Submodule providing the `GetColumns` trait.

use crate::{
    Columns, GetColumn, HasTableAddition, InsertableTableModel, MayGetColumn, TableAddition,
    TrySetColumn, Typed, TypedColumn,
};
use diesel::associations::HasTable;
use tuplities::prelude::*;

#[diesel_builders_macros::impl_get_columns]
/// Trait indicating a builder can get multiple columns.
pub trait GetColumns<CS: Columns> {
    /// Get the values of the specified columns.
    fn get_columns(&self) -> CS::Type;
    /// Get the references of the specified columns.
    fn get_columns_ref(&self) -> <CS::Type as TupleRef>::Ref<'_>;
}

/// Variant of `GetColumns` for n-uples.
pub trait TupleGetColumns<CS: Typed<Type: FlattenNestedTuple>> {
    /// Get the values of the specified columns as an n-uple.
    fn tuple_get_columns(&self) -> <<CS as Typed>::Type as FlattenNestedTuple>::Flattened;
}

impl TupleGetColumns<()> for () {
    #[inline]
    fn tuple_get_columns(&self) -> () {}
}

impl<T1, C1> TupleGetColumns<(C1,)> for (T1,)
where
    T1: GetColumn<C1>,
    C1: TypedColumn,
{
    #[inline]
    fn tuple_get_columns(&self) -> <<(C1,) as Typed>::Type as FlattenNestedTuple>::Flattened {
        (self.0.get_column(),)
    }
}

impl<Chead, CTail, Thead, TTail> TupleGetColumns<(Chead, CTail)> for (Thead, TTail)
where
    Chead: TypedColumn,
    CTail: Typed<Type: FlattenNestedTuple>,
    (Chead::Type, CTail::Type): FlattenNestedTuple<Flattened: TupleRef>,
    Thead: GetColumn<Chead>,
    TTail: TupleGetColumns<CTail>,
    <CTail::Type as FlattenNestedTuple>::Flattened: TuplePushFront<
            Chead::Type,
            Output = <(Chead::Type, CTail::Type) as FlattenNestedTuple>::Flattened,
        >,
{
    #[inline]
    fn tuple_get_columns(
        &self,
    ) -> <<(Chead, CTail) as Typed>::Type as FlattenNestedTuple>::Flattened {
        let head = self.0.get_column();
        let tail = self.1.tuple_get_columns();
        tail.push_front(head)
    }
}

#[diesel_builders_macros::impl_may_get_columns]
/// Trait indicating a builder which may get multiple columns.
pub trait MayGetColumns<CS: Columns> {
    /// May get the references of the specified columns.
    fn may_get_columns_ref(
        &self,
    ) -> <<CS::Type as TupleRef>::Ref<'_> as IntoTupleOption>::IntoOptions;
    /// May get the owned values of the specified columns.
    fn may_get_columns(&self) -> <CS::Type as IntoTupleOption>::IntoOptions;
}

/// Variant of `MayGetColumns` for n-uples.
pub trait TupleMayGetColumns<CS: Typed<Type: FlattenNestedTuple<Flattened: IntoTupleOption>>> {
    /// May get the values of the specified columns as an n-uple.
    fn tuple_may_get_columns(
        &self,
    ) -> <<<CS as Typed>::Type as FlattenNestedTuple>::Flattened as IntoTupleOption>::IntoOptions;
}

impl TupleMayGetColumns<()> for () {
    #[inline]
    fn tuple_may_get_columns(&self) -> () {}
}

impl<T1, C1> TupleMayGetColumns<(C1,)> for (T1,)
where
    T1: MayGetColumn<C1>,
    C1: TypedColumn,
{
    #[inline]
    fn tuple_may_get_columns(
        &self,
    ) -> <<<(C1,) as Typed>::Type as FlattenNestedTuple>::Flattened as IntoTupleOption>::IntoOptions
    {
        (self.0.may_get_column(),)
    }
}

impl<Chead, CTail, Thead, TTail> TupleMayGetColumns<(Chead, CTail)> for (Thead, TTail)
where
    Chead: TypedColumn,
    CTail: Typed<Type: FlattenNestedTuple<Flattened: IntoTupleOption>>,
    (Chead::Type, CTail::Type): FlattenNestedTuple<Flattened: IntoTupleOption>,
    Thead: MayGetColumn<Chead>,
    TTail: TupleMayGetColumns<CTail>,
    <<CTail::Type as FlattenNestedTuple>::Flattened as IntoTupleOption>::IntoOptions: TuplePushFront<
            Option<Chead::Type>,
            Output = <<(Chead::Type, CTail::Type) as FlattenNestedTuple>::Flattened as IntoTupleOption>::IntoOptions,
        >,
{
    #[inline]
    fn tuple_may_get_columns(
        &self,
    ) -> <<<(Chead, CTail) as Typed>::Type as FlattenNestedTuple>::Flattened as IntoTupleOption>::IntoOptions
    {
        let head = self.0.may_get_column();
        let tail = self.1.tuple_may_get_columns();
        tail.push_front(head)
    }
}

#[diesel_builders_macros::impl_set_columns]
/// Trait indicating a builder can set multiple columns.
pub trait SetColumns<CS: Columns> {
    /// Set the values of the specified columns.
    fn set_columns(&mut self, values: <CS as Typed>::Type) -> &mut Self;
}

#[diesel_builders_macros::impl_may_set_columns]
/// Trait indicating a builder which may set multiple columns.
pub trait MaySetColumns<CS: Columns> {
    /// May set the values of the specified columns.
    fn may_set_columns(
        &mut self,
        values: <<CS as Typed>::Type as IntoTupleOption>::IntoOptions,
    ) -> &mut Self;
}

#[diesel_builders_macros::impl_try_set_columns]
/// Trait indicating a builder can fallibly set multiple columns.
pub trait TrySetColumns<Error, CS: Typed<Type: TupleRef>> {
    /// Attempt to set the values of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if any column cannot be set.
    fn try_set_columns(&mut self, values: <CS as Typed>::Type) -> Result<&mut Self, Error>;
}

#[diesel_builders_macros::impl_try_set_columns_collection]
/// Trait indicating a builder can fallibly set multiple columns.
pub trait TrySetColumnsCollection<Error, ColumnsCollection: Typed> {
    /// Attempt to set the values of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if any column cannot be set.
    fn try_set_columns_collection(
        &mut self,
        values: <ColumnsCollection as Typed>::Type,
    ) -> Result<&mut Self, Error>;
}

#[diesel_builders_macros::impl_try_may_set_columns]
/// Trait indicating a builder which may try to set multiple columns.
pub trait TryMaySetColumns<Error, CS: Columns> {
    /// Attempt to set the values of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if any column cannot be set.
    fn try_may_set_columns(
        &mut self,
        values: <<CS as Typed>::Type as IntoTupleOption>::IntoOptions,
    ) -> Result<&mut Self, Error>;
}

/// Trait indicating a builder can try to set multiple homogeneous
/// columns.
pub trait TrySetHomogeneous<Error, Type, CS: Typed>: TrySetColumnsCollection<Error, CS> {
    /// Attempt to set the values of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if any column cannot be set.
    fn try_set_homogeneous(&mut self, value: Type) -> Result<&mut Self, Error>;
}

impl<Error, T, Type: core::fmt::Debug + Clone, ColumnsCollection: Typed<Type: TupleReplicate<Type>>>
    TrySetHomogeneous<Error, Type, ColumnsCollection> for T
where
    T: TrySetColumnsCollection<Error, ColumnsCollection>,
{
    #[inline]
    fn try_set_homogeneous(&mut self, value: Type) -> Result<&mut Self, Error> {
        let replicates =
            <<ColumnsCollection as Typed>::Type as TupleReplicate<Type>>::tuple_replicate(value);
        <T as TrySetColumnsCollection<Error, ColumnsCollection>>::try_set_columns_collection(
            self, replicates,
        )
    }
}
