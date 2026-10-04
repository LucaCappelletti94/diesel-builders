//! Shared interfaces for atomic fallible column mutations.
//!
//! A group prepares every present leaf before any of them is applied, so a
//! later rejected leaf leaves the earlier values untouched and restores the
//! consumed inputs for the whole group on failure.
//!
//! A `MutationContext` names the side builder slot that an attachment
//! replaces, so preparation skips the old builder that the attachment
//! supersedes.

use tuplities::prelude::IntoNestedTupleOption;

use crate::{
    NestedColumns, OptionalRef, SetColumn, TableExt, TypedColumn, TypedNestedTupleCollection,
    ValidateColumn,
};

/// Tracks the side builder slot that an attachment replaces during a mutation.
#[derive(Default)]
pub(crate) struct MutationContext {
    /// The superseded side builder slot as `(owner, mandatory, index)`.
    excluded: Option<(*const (), bool, usize)>,
}

impl MutationContext {
    /// Excludes the mandatory side builder slot `index` owned by `owner`.
    #[inline]
    pub(crate) fn excluding_mandatory(owner: *const (), index: usize) -> Self {
        Self { excluded: Some((owner, true, index)) }
    }

    /// Excludes the discretionary side builder slot `index` owned by `owner`.
    #[inline]
    pub(crate) fn excluding_discretionary(owner: *const (), index: usize) -> Self {
        Self { excluded: Some((owner, false, index)) }
    }

    /// Reports whether the mandatory side builder slot `index` owned by `owner`
    /// is excluded.
    #[inline]
    pub(crate) fn excludes_mandatory(&self, owner: *const (), index: usize) -> bool {
        self.excluded.is_some_and(|(excluded_owner, mandatory, excluded_index)| {
            excluded_owner == owner && mandatory && excluded_index == index
        })
    }

    /// Reports whether the discretionary side builder slot `index` owned by
    /// `owner` is excluded.
    #[inline]
    pub(crate) fn excludes_discretionary(&self, owner: *const (), index: usize) -> bool {
        self.excluded.is_some_and(|(excluded_owner, mandatory, excluded_index)| {
            excluded_owner == owner && !mandatory && excluded_index == index
        })
    }
}

/// Borrows or consumes a prepared column value.
pub(crate) trait ColumnInput<C: TypedColumn> {
    /// Borrows the prepared column value.
    fn value(&self) -> &C::ColumnType;
    /// Consumes the prepared column value.
    fn into_value(self) -> C::ColumnType;
}

/// Prepares one column value before it is applied.
pub(crate) trait PrepareColumn<C: TypedColumn>: ValidateColumn<C> {
    /// The prepared, validated value.
    type Prepared: ColumnInput<C>;
    /// Validates the value, returning the prepared value or the consumed input
    /// and error.
    fn prepare_column(
        &self,
        value: C::ColumnType,
        context: &MutationContext,
    ) -> Result<Self::Prepared, (C::ColumnType, Self::Error)>;
    /// Applies the prepared value to the column.
    fn apply_column(&mut self, prepared: Self::Prepared);
}

/// Prepares a column group before any of its leaves are applied.
pub(crate) trait PrepareColumns<E, CS: NestedColumns> {
    /// The prepared group value.
    type Prepared;
    /// Prepares every column, restoring the raw inputs on failure.
    fn prepare_columns(
        &self,
        values: CS::NestedTupleColumnType,
        context: &MutationContext,
    ) -> Result<Self::Prepared, (CS::NestedTupleColumnType, E)>;
    /// Restores the raw input values from a prepared group.
    fn restore_columns(prepared: Self::Prepared) -> CS::NestedTupleColumnType;
    /// Applies the prepared group.
    fn apply_columns(&mut self, prepared: Self::Prepared);
}

/// Prepares a column group that may hold absent leaves.
pub(crate) trait PrepareOptionalColumns<E, CS: NestedColumns> {
    /// The prepared optional group value.
    type Prepared;
    /// Prepares the optional group, restoring the raw inputs on failure.
    fn prepare_optional_columns(
        &self,
        values: <CS::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
        context: &MutationContext,
    ) -> Result<
        Self::Prepared,
        (<CS::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions, E),
    >;
    /// Applies the prepared optional group.
    fn apply_optional_columns(&mut self, prepared: Self::Prepared);
}

/// Prepares one value broadcast to a homogeneous column group.
pub(crate) trait PrepareHomogeneous<E, Type, CS: NestedColumns> {
    /// The prepared homogeneous group value.
    type Prepared;
    /// Prepares the value for every column of the group.
    fn prepare_homogeneous(
        &self,
        value: &impl OptionalRef<Type>,
        context: &MutationContext,
    ) -> Result<Self::Prepared, E>;
    /// Applies the prepared homogeneous group.
    fn apply_homogeneous(&mut self, prepared: Self::Prepared);
}

/// Prepares a collection of nested column groups.
pub(crate) trait PrepareColumnsCollection<E, NCC: TypedNestedTupleCollection> {
    /// The prepared collection value.
    type Prepared;
    /// Prepares every nested group, restoring the raw inputs on failure.
    fn prepare_columns_collection(
        &self,
        values: NCC::NestedCollectionType,
        context: &MutationContext,
    ) -> Result<Self::Prepared, (NCC::NestedCollectionType, E)>;
    /// Applies the prepared collection.
    fn apply_columns_collection(&mut self, prepared: Self::Prepared);
}

/// An owned candidate value prepared for a raw tuple setter.
pub(crate) struct PreparedColumnValue<C: TypedColumn> {
    /// The owned column value.
    value: C::ColumnType,
}

impl<C: TypedColumn> ColumnInput<C> for PreparedColumnValue<C> {
    #[inline]
    fn value(&self) -> &C::ColumnType {
        &self.value
    }

    #[inline]
    fn into_value(self) -> C::ColumnType {
        self.value
    }
}

/// Emits the identical leaf `PrepareColumn` body for each raw tuple arity.
/// These are non-recursive leaf impls that validate a borrowed nullable
/// inner value once, restore the consumed value on failure and apply the raw
/// `SetColumn` without revalidation, ignoring the mutation context.
macro_rules! impl_prepare_column_for_tuple {
    ($( impl[$($generics:ident),+] for $tuple:ty ),+ $(,)?) => {
        $(
            impl<$($generics),+, C> PrepareColumn<C> for $tuple
            where
                Self: SetColumn<C> + ValidateColumn<C>,
                C: TypedColumn,
            {
                type Prepared = PreparedColumnValue<C>;

                #[inline]
                fn prepare_column(
                    &self,
                    value: C::ColumnType,
                    _context: &MutationContext,
                ) -> Result<Self::Prepared, (C::ColumnType, Self::Error)> {
                    let prepared = PreparedColumnValue::<C> { value };
                    if let Some(value_ref) = prepared.value().as_optional_ref() {
                        match <Self as ValidateColumn<C>>::validate_column(value_ref) {
                            Ok(()) => {}
                            Err(error) => return Err((prepared.into_value(), error)),
                        }
                    }
                    Ok(prepared)
                }

                #[inline]
                fn apply_column(&mut self, prepared: Self::Prepared) {
                    <Self as SetColumn<C>>::set_column(self, prepared.into_value());
                }
            }
        )+
    };
}

impl_prepare_column_for_tuple! {
    impl[T] for (T,),
    impl[Head, Tail] for (Head, Tail),
}

impl<T, E> PrepareColumns<E, ()> for T {
    type Prepared = ();

    #[inline]
    fn prepare_columns(
        &self,
        _values: (),
        _context: &MutationContext,
    ) -> Result<Self::Prepared, ((), E)> {
        Ok(())
    }

    #[inline]
    fn restore_columns(_prepared: Self::Prepared) {}

    #[inline]
    fn apply_columns(&mut self, _prepared: Self::Prepared) {}
}

impl<C1, T, E> PrepareColumns<E, (C1,)> for T
where
    T: PrepareColumn<C1>,
    C1: TypedColumn<Table: TableExt>,
    E: From<<T as ValidateColumn<C1>>::Error>,
{
    type Prepared = <T as PrepareColumn<C1>>::Prepared;

    #[inline]
    fn prepare_columns(
        &self,
        values: (C1::ColumnType,),
        context: &MutationContext,
    ) -> Result<Self::Prepared, ((C1::ColumnType,), E)> {
        <T as PrepareColumn<C1>>::prepare_column(self, values.0, context)
            .map_err(|(value, error)| ((value,), error.into()))
    }

    #[inline]
    fn restore_columns(prepared: Self::Prepared) -> (C1::ColumnType,) {
        (prepared.into_value(),)
    }

    #[inline]
    fn apply_columns(&mut self, prepared: Self::Prepared) {
        <T as PrepareColumn<C1>>::apply_column(self, prepared);
    }
}

impl<CHead, CTail, T, E> PrepareColumns<E, (CHead, CTail)> for T
where
    CHead: TypedColumn,
    CTail: NestedColumns,
    (CHead, CTail):
        NestedColumns<NestedTupleColumnType = (CHead::ColumnType, CTail::NestedTupleColumnType)>,
    T: PrepareColumn<CHead> + PrepareColumns<E, CTail>,
    E: From<<T as ValidateColumn<CHead>>::Error>,
{
    type Prepared =
        (<T as PrepareColumn<CHead>>::Prepared, <T as PrepareColumns<E, CTail>>::Prepared);

    #[inline]
    fn prepare_columns(
        &self,
        (head, tail): (CHead::ColumnType, CTail::NestedTupleColumnType),
        context: &MutationContext,
    ) -> Result<Self::Prepared, ((CHead::ColumnType, CTail::NestedTupleColumnType), E)> {
        let head_prepared = match <T as PrepareColumn<CHead>>::prepare_column(self, head, context) {
            Ok(prepared) => prepared,
            Err((head, error)) => return Err(((head, tail), error.into())),
        };
        let tail_prepared =
            match <T as PrepareColumns<E, CTail>>::prepare_columns(self, tail, context) {
                Ok(prepared) => prepared,
                Err((tail, error)) => {
                    return Err(((head_prepared.into_value(), tail), error));
                }
            };
        Ok((head_prepared, tail_prepared))
    }

    #[inline]
    fn restore_columns(
        prepared: Self::Prepared,
    ) -> (CHead::ColumnType, CTail::NestedTupleColumnType) {
        let (head, tail) = prepared;
        (head.into_value(), <T as PrepareColumns<E, CTail>>::restore_columns(tail))
    }

    #[inline]
    fn apply_columns(&mut self, prepared: Self::Prepared) {
        let (head, tail) = prepared;
        <T as PrepareColumn<CHead>>::apply_column(self, head);
        <T as PrepareColumns<E, CTail>>::apply_columns(self, tail);
    }
}

impl<T, E> PrepareOptionalColumns<E, ()> for T {
    type Prepared = ();

    #[inline]
    fn prepare_optional_columns(
        &self,
        _values: (),
        _context: &MutationContext,
    ) -> Result<Self::Prepared, ((), E)> {
        Ok(())
    }

    #[inline]
    fn apply_optional_columns(&mut self, _prepared: Self::Prepared) {}
}

impl<C1, T, E> PrepareOptionalColumns<E, (C1,)> for T
where
    T: PrepareColumn<C1>,
    C1: TypedColumn<Table: TableExt>,
    E: From<<T as ValidateColumn<C1>>::Error>,
{
    type Prepared = Option<<T as PrepareColumn<C1>>::Prepared>;

    #[inline]
    fn prepare_optional_columns(
        &self,
        (value,): (Option<C1::ColumnType>,),
        context: &MutationContext,
    ) -> Result<Self::Prepared, ((Option<C1::ColumnType>,), E)> {
        match value {
            Some(value) => {
                match <T as PrepareColumn<C1>>::prepare_column(self, value, context) {
                    Ok(prepared) => Ok(Some(prepared)),
                    Err((value, error)) => Err(((Some(value),), error.into())),
                }
            }
            None => Ok(None),
        }
    }

    #[inline]
    fn apply_optional_columns(&mut self, prepared: Self::Prepared) {
        if let Some(prepared) = prepared {
            <T as PrepareColumn<C1>>::apply_column(self, prepared);
        }
    }
}

impl<CHead, CTail, T, E> PrepareOptionalColumns<E, (CHead, CTail)> for T
where
    CHead: TypedColumn,
    CTail: NestedColumns,
    (CHead, CTail):
        NestedColumns<NestedTupleColumnType = (CHead::ColumnType, CTail::NestedTupleColumnType)>,
    T: PrepareColumn<CHead> + PrepareOptionalColumns<E, CTail>,
    E: From<<T as ValidateColumn<CHead>>::Error>,
{
    type Prepared = (
        Option<<T as PrepareColumn<CHead>>::Prepared>,
        <T as PrepareOptionalColumns<E, CTail>>::Prepared,
    );

    #[inline]
    fn prepare_optional_columns(
        &self,
        (head, tail): (
            Option<CHead::ColumnType>,
            <CTail::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
        ),
        context: &MutationContext,
    ) -> Result<
        Self::Prepared,
        (
            (
                Option<CHead::ColumnType>,
                <CTail::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
            ),
            E,
        ),
    > {
        let head_prepared = match head {
            Some(value) => {
                Some(match <T as PrepareColumn<CHead>>::prepare_column(self, value, context) {
                    Ok(prepared) => prepared,
                    Err((value, error)) => return Err(((Some(value), tail), error.into())),
                })
            }
            None => None,
        };
        let tail_prepared = match <T as PrepareOptionalColumns<E, CTail>>::prepare_optional_columns(
            self, tail, context,
        ) {
            Ok(prepared) => prepared,
            Err((tail, error)) => {
                return Err(((head_prepared.map(ColumnInput::into_value), tail), error));
            }
        };
        Ok((head_prepared, tail_prepared))
    }

    #[inline]
    fn apply_optional_columns(&mut self, prepared: Self::Prepared) {
        let (head, tail) = prepared;
        if let Some(prepared) = head {
            <T as PrepareColumn<CHead>>::apply_column(self, prepared);
        }
        <T as PrepareOptionalColumns<E, CTail>>::apply_optional_columns(self, tail);
    }
}

impl<T, E, Type> PrepareHomogeneous<E, Type, ()> for T {
    type Prepared = ();

    #[inline]
    fn prepare_homogeneous(
        &self,
        _value: &impl OptionalRef<Type>,
        _context: &MutationContext,
    ) -> Result<Self::Prepared, E> {
        Ok(())
    }

    #[inline]
    fn apply_homogeneous(&mut self, _prepared: Self::Prepared) {}
}

impl<Type, C1, T, E> PrepareHomogeneous<E, Type, (C1,)> for T
where
    Type: Clone,
    T: PrepareColumn<C1>,
    C1: TypedColumn<ColumnType: From<Type>, Table: TableExt>,
    E: From<<T as ValidateColumn<C1>>::Error>,
{
    type Prepared = Option<<T as PrepareColumn<C1>>::Prepared>;

    #[inline]
    fn prepare_homogeneous(
        &self,
        value: &impl OptionalRef<Type>,
        context: &MutationContext,
    ) -> Result<Self::Prepared, E> {
        match value.as_optional_ref() {
            Some(value) => {
                let converted: C1::ColumnType = value.clone().into();
                <T as PrepareColumn<C1>>::prepare_column(self, converted, context)
                    .map(Some)
                    .map_err(|(_, error)| error.into())
            }
            None => Ok(None),
        }
    }

    #[inline]
    fn apply_homogeneous(&mut self, prepared: Self::Prepared) {
        if let Some(prepared) = prepared {
            <T as PrepareColumn<C1>>::apply_column(self, prepared);
        }
    }
}

impl<Type, CHead, CTail, T, E> PrepareHomogeneous<E, Type, (CHead, CTail)> for T
where
    Type: Clone,
    CHead: TypedColumn,
    CHead::ColumnType: From<Type>,
    CTail: NestedColumns,
    (CHead, CTail):
        NestedColumns<NestedTupleColumnType = (CHead::ColumnType, CTail::NestedTupleColumnType)>,
    T: PrepareColumn<CHead> + PrepareHomogeneous<E, Type, CTail>,
    E: From<<T as ValidateColumn<CHead>>::Error>,
{
    type Prepared = (
        Option<<T as PrepareColumn<CHead>>::Prepared>,
        <T as PrepareHomogeneous<E, Type, CTail>>::Prepared,
    );

    #[inline]
    fn prepare_homogeneous(
        &self,
        value: &impl OptionalRef<Type>,
        context: &MutationContext,
    ) -> Result<Self::Prepared, E> {
        let head_prepared = if let Some(value) = value.as_optional_ref() {
            let converted: CHead::ColumnType = value.clone().into();
            Some(
                <T as PrepareColumn<CHead>>::prepare_column(self, converted, context)
                    .map_err(|(_, error)| error)?,
            )
        } else {
            None
        };
        let tail_prepared =
            <T as PrepareHomogeneous<E, Type, CTail>>::prepare_homogeneous(self, value, context)?;
        Ok((head_prepared, tail_prepared))
    }

    #[inline]
    fn apply_homogeneous(&mut self, prepared: Self::Prepared) {
        let (head, tail) = prepared;
        if let Some(prepared) = head {
            <T as PrepareColumn<CHead>>::apply_column(self, prepared);
        }
        <T as PrepareHomogeneous<E, Type, CTail>>::apply_homogeneous(self, tail);
    }
}

impl<T, E> PrepareColumnsCollection<E, ()> for T {
    type Prepared = ();

    #[inline]
    fn prepare_columns_collection(
        &self,
        _values: (),
        _context: &MutationContext,
    ) -> Result<Self::Prepared, ((), E)> {
        Ok(())
    }

    #[inline]
    fn apply_columns_collection(&mut self, _prepared: Self::Prepared) {}
}

impl<C1, T, E> PrepareColumnsCollection<E, (C1,)> for T
where
    C1: NestedColumns,
    T: PrepareColumns<E, C1>,
{
    type Prepared = <T as PrepareColumns<E, C1>>::Prepared;

    #[inline]
    fn prepare_columns_collection(
        &self,
        values: (C1::NestedTupleColumnType,),
        context: &MutationContext,
    ) -> Result<Self::Prepared, ((C1::NestedTupleColumnType,), E)> {
        <T as PrepareColumns<E, C1>>::prepare_columns(self, values.0, context)
            .map_err(|(values, error)| ((values,), error))
    }

    #[inline]
    fn apply_columns_collection(&mut self, prepared: Self::Prepared) {
        <T as PrepareColumns<E, C1>>::apply_columns(self, prepared);
    }
}

impl<CHead, CTail, T, E> PrepareColumnsCollection<E, (CHead, CTail)> for T
where
    CHead: NestedColumns,
    CTail: TypedNestedTupleCollection,
    (CHead, CTail): TypedNestedTupleCollection<
        NestedCollectionType = (
            CHead::NestedTupleColumnType,
            <CTail as TypedNestedTupleCollection>::NestedCollectionType,
        ),
    >,
    T: PrepareColumns<E, CHead> + PrepareColumnsCollection<E, CTail>,
{
    type Prepared = (
        <T as PrepareColumns<E, CHead>>::Prepared,
        <T as PrepareColumnsCollection<E, CTail>>::Prepared,
    );

    #[inline]
    fn prepare_columns_collection(
        &self,
        (head, tail): (
            CHead::NestedTupleColumnType,
            <CTail as TypedNestedTupleCollection>::NestedCollectionType,
        ),
        context: &MutationContext,
    ) -> Result<
        Self::Prepared,
        (
            (
                CHead::NestedTupleColumnType,
                <CTail as TypedNestedTupleCollection>::NestedCollectionType,
            ),
            E,
        ),
    > {
        let head_prepared =
            match <T as PrepareColumns<E, CHead>>::prepare_columns(self, head, context) {
                Ok(prepared) => prepared,
                Err((head, error)) => return Err(((head, tail), error)),
            };
        let tail_prepared =
            match <T as PrepareColumnsCollection<E, CTail>>::prepare_columns_collection(
                self, tail, context,
            ) {
                Ok(prepared) => prepared,
                Err((tail, error)) => {
                    return Err((
                        (<T as PrepareColumns<E, CHead>>::restore_columns(head_prepared), tail),
                        error,
                    ));
                }
            };
        Ok((head_prepared, tail_prepared))
    }

    #[inline]
    fn apply_columns_collection(&mut self, prepared: Self::Prepared) {
        let (head, tail) = prepared;
        <T as PrepareColumns<E, CHead>>::apply_columns(self, head);
        <T as PrepareColumnsCollection<E, CTail>>::apply_columns_collection(self, tail);
    }
}
