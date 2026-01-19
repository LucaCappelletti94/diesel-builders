//! Blanket implementations of `IterForeignKey` for tuples.

use tuplities::prelude::{IntoNestedTupleOption, NestedTupleRef};

use crate::{
    IterDynForeignKeys, IterForeignKeys, TryGetDynamicColumns, TypedNestedTuple,
    builder_error::DynamicColumnError,
    columns::{HasNestedDynColumns, NestedDynColumns, NonEmptyNestedProjection},
};

impl<NestedIdx> IterForeignKeys<NestedIdx> for ()
where
    NestedIdx: HasNestedDynColumns + NonEmptyNestedProjection,
{
    fn iter_foreign_key_columns()
    -> impl Iterator<Item = <NestedIdx as HasNestedDynColumns>::NestedDynColumns> {
        std::iter::empty()
    }

    fn iter_match_simple<'a>(
        &'a self,
    ) -> impl Iterator<
        Item = <<<NestedIdx as TypedNestedTuple>::NestedTupleValueType as NestedTupleRef>::Ref<'a> as IntoNestedTupleOption>::IntoOptions,
    >
    where
        NestedIdx: 'a,
    {
        std::iter::empty()
    }
}

impl<NestedIdx, T> IterForeignKeys<NestedIdx> for (T,)
where
    NestedIdx: HasNestedDynColumns + NonEmptyNestedProjection,
    T: IterForeignKeys<NestedIdx>,
{
    fn iter_foreign_key_columns()
    -> impl Iterator<Item = <NestedIdx as HasNestedDynColumns>::NestedDynColumns> {
        T::iter_foreign_key_columns()
    }

    fn iter_match_simple<'a>(
        &'a self,
    ) -> impl Iterator<
        Item = <<<NestedIdx as TypedNestedTuple>::NestedTupleValueType as NestedTupleRef>::Ref<'a> as IntoNestedTupleOption>::IntoOptions,
    >
    where
        NestedIdx: 'a,
    {
        self.0.iter_match_simple()
    }
}

impl<NestedIdx, Head, Tail> IterForeignKeys<NestedIdx> for (Head, Tail)
where
    NestedIdx: HasNestedDynColumns + NonEmptyNestedProjection,
    Head: IterForeignKeys<NestedIdx>,
    Tail: IterForeignKeys<NestedIdx>,
{
    fn iter_foreign_key_columns()
    -> impl Iterator<Item = <NestedIdx as HasNestedDynColumns>::NestedDynColumns> {
        Head::iter_foreign_key_columns().chain(Tail::iter_foreign_key_columns())
    }

    fn iter_match_simple<'a>(
        &'a self,
    ) -> impl Iterator<
        Item = <<<NestedIdx as TypedNestedTuple>::NestedTupleValueType as NestedTupleRef>::Ref<'a> as IntoNestedTupleOption>::IntoOptions,
    >
    where
        NestedIdx: 'a,
    {
        self.0.iter_match_simple().chain(self.1.iter_match_simple())
    }
}

impl<DynIdx, T> IterDynForeignKeys<DynIdx> for (T,)
where
    DynIdx: NestedDynColumns,
    T: IterDynForeignKeys<DynIdx>,
    Self: TryGetDynamicColumns,
{
    fn iter_foreign_key_dyn_columns(
        index: DynIdx,
    ) -> Result<impl Iterator<Item = DynIdx>, DynamicColumnError> {
        T::iter_foreign_key_dyn_columns(index)
    }
}

impl<DynIdx, Head, Tail> IterDynForeignKeys<DynIdx> for (Head, Tail)
where
    DynIdx: NestedDynColumns + Clone,
    Head: IterDynForeignKeys<DynIdx>,
    Tail: IterDynForeignKeys<DynIdx>,
    Self: TryGetDynamicColumns,
{
    fn iter_foreign_key_dyn_columns(
        index: DynIdx,
    ) -> Result<impl Iterator<Item = DynIdx>, DynamicColumnError> {
        Ok(Head::iter_foreign_key_dyn_columns(index.clone())?
            .chain(Tail::iter_foreign_key_dyn_columns(index)?))
    }
}

impl<DynIdx, T> IterDynForeignKeys<DynIdx> for Option<T>
where
    DynIdx: NestedDynColumns,
    T: IterDynForeignKeys<DynIdx>,
    Self: TryGetDynamicColumns,
{
    fn iter_foreign_key_dyn_columns(
        index: DynIdx,
    ) -> Result<impl Iterator<Item = DynIdx>, DynamicColumnError> {
        T::iter_foreign_key_dyn_columns(index)
    }
}
