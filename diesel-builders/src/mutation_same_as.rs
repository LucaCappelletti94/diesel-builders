//! Leaf and group traits for atomic fallible preparation of mandatory and
//! discretionary same-as column sets.
//!
//! A group prepares every present leaf before any of them is applied, so a
//! later rejected leaf leaves the earlier side builders untouched. A null
//! source or an absent side builder produces no prepared leaf and no
//! allocation. An exclusion context skips side builders whose owner is being
//! replaced.
//!
//! The mandatory and discretionary families are structurally identical, so
//! both are emitted from the `impl_same_as_column_preparation` macro, keyed
//! by their index, generated trait and method names.

use crate::{
    DiscretionarySameAsIndex, MandatorySameAsIndex, OptionalRef, TypedColumn,
    columns::NestedColumns, mutation::MutationContext,
};

/// Emits the mandatory and discretionary same-as preparation trait and impl
/// families.
macro_rules! impl_same_as_column_preparation {
    (
        kind = $kind:literal,
        index = $index:ident,
        leaf_trait = $leaf_trait:ident,
        has_method = $has_method:ident,
        prepare_leaf_method = $prepare_leaf_method:ident,
        apply_leaf_method = $apply_leaf_method:ident,
        group_trait = $group_trait:ident,
        prepare_group_method = $prepare_group_method:ident,
        apply_group_method = $apply_group_method:ident $(,)?
    ) => {
        #[doc = concat!(
            "Prepares one column of a ", $kind, " same-as relationship on a checked ",
            "builder, separating validation from application."
        )]
        pub(crate) trait $leaf_trait<
            Key: $index,
            C: TypedColumn<Table = Key::ReferencedTable>,
        > {
            /// The associated error type for the operation.
            type Error;

            #[doc = concat!(
                "The prepared value applied by `", stringify!($apply_leaf_method), "`."
            )]
            type Prepared;

            #[doc = concat!(
                "Reports whether the ", $kind,
                " side builder a value would propagate to is present and not excluded."
            )]
            fn $has_method(&self, context: &MutationContext) -> bool;

            #[doc = concat!(
                "Validates and prepares one column value for the ", $kind,
                " same-as relationship.\n\n# Errors\n\nReturns the converted value ",
                "with its error if validation fails."
            )]
            fn $prepare_leaf_method(
                &self,
                value: C::ColumnType,
                context: &MutationContext,
            ) -> Result<Self::Prepared, (C::ColumnType, Self::Error)>;

            #[doc = concat!(
                "Applies a prepared column value to the ", $kind, " side builder."
            )]
            fn $apply_leaf_method(&mut self, prepared: Self::Prepared);
        }

        #[doc = concat!(
            "Prepares a group of ", $kind, " same-as columns from one borrowed source ",
            "value before any of them is applied."
        )]
        pub(crate) trait $group_trait<E, Type, Keys: NestedColumns, CS: NestedColumns> {
            #[doc = concat!(
                "The prepared group applied by `", stringify!($apply_group_method), "`."
            )]
            type Prepared;

            #[doc = concat!(
                "Prepares every present leaf of the group.\n\n# Errors\n\nReturns an error ",
                "if a leaf value fails validation."
            )]
            fn $prepare_group_method(
                &self,
                value: &impl OptionalRef<Type>,
                context: &MutationContext,
            ) -> Result<Self::Prepared, E>;

            #[doc = concat!(
                "Applies a prepared group to the ", $kind, " side builders."
            )]
            fn $apply_group_method(&mut self, prepared: Self::Prepared);
        }

        impl<T, Type, E> $group_trait<E, Type, (), ()> for T {
            type Prepared = ();

            #[inline]
            fn $prepare_group_method(
                &self,
                _value: &impl OptionalRef<Type>,
                _context: &MutationContext,
            ) -> Result<Self::Prepared, E> {
                Ok(())
            }

            #[inline]
            fn $apply_group_method(&mut self, _prepared: Self::Prepared) {}
        }

        impl<
            Type: Clone,
            E,
            T,
            Key: $index,
            C: TypedColumn<Table = Key::ReferencedTable>,
        > $group_trait<E, Type, (Key,), (C,)> for T
        where
            C::ColumnType: From<Type>,
            T: $leaf_trait<Key, C>,
            E: From<<T as $leaf_trait<Key, C>>::Error>,
        {
            type Prepared = Option<<T as $leaf_trait<Key, C>>::Prepared>;

            #[inline]
            fn $prepare_group_method(
                &self,
                value: &impl OptionalRef<Type>,
                context: &MutationContext,
            ) -> Result<Self::Prepared, E> {
                let Some(value) = value.as_optional_ref() else {
                    return Ok(None);
                };
                if !<T as $leaf_trait<Key, C>>::$has_method(self, context) {
                    return Ok(None);
                }
                let converted = <C::ColumnType as From<Type>>::from(value.clone());
                let prepared = <T as $leaf_trait<Key, C>>::$prepare_leaf_method(
                    self, converted, context,
                )
                .map_err(|(_, error)| error)?;
                Ok(Some(prepared))
            }

            #[inline]
            fn $apply_group_method(&mut self, prepared: Self::Prepared) {
                if let Some(prepared) = prepared {
                    <T as $leaf_trait<Key, C>>::$apply_leaf_method(self, prepared);
                }
            }
        }

        impl<
            Type: Clone,
            E,
            T,
            KeysHead: $index,
            KeysTail: NestedColumns,
            CHead: TypedColumn<Table = KeysHead::ReferencedTable>,
            CTail: NestedColumns,
        > $group_trait<E, Type, (KeysHead, KeysTail), (CHead, CTail)> for T
        where
            (KeysHead, KeysTail): NestedColumns,
            (CHead, CTail): NestedColumns,
            CHead::ColumnType: From<Type>,
            T: $leaf_trait<KeysHead, CHead> + $group_trait<E, Type, KeysTail, CTail>,
            E: From<<T as $leaf_trait<KeysHead, CHead>>::Error>,
        {
            type Prepared = (
                Option<<T as $leaf_trait<KeysHead, CHead>>::Prepared>,
                <T as $group_trait<E, Type, KeysTail, CTail>>::Prepared,
            );

            #[inline]
            fn $prepare_group_method(
                &self,
                value: &impl OptionalRef<Type>,
                context: &MutationContext,
            ) -> Result<Self::Prepared, E> {
                let Some(value) = value.as_optional_ref() else {
                    let tail = <T as $group_trait<E, Type, KeysTail, CTail>>::$prepare_group_method(
                        self, value, context,
                    )?;
                    return Ok((None, tail));
                };
                let head =
                    if <T as $leaf_trait<KeysHead, CHead>>::$has_method(self, context) {
                        let converted = <CHead::ColumnType as From<Type>>::from(value.clone());
                        let prepared = <T as $leaf_trait<KeysHead, CHead>>::$prepare_leaf_method(
                            self, converted, context,
                        )
                        .map_err(|(_, error)| error)?;
                        Some(prepared)
                    } else {
                        None
                    };
                let tail =
                    <T as $group_trait<E, Type, KeysTail, CTail>>::$prepare_group_method(
                        self, value, context,
                    )?;
                Ok((head, tail))
            }

            #[inline]
            fn $apply_group_method(&mut self, prepared: Self::Prepared) {
                let (head, tail) = prepared;
                if let Some(head) = head {
                    <T as $leaf_trait<KeysHead, CHead>>::$apply_leaf_method(self, head);
                }
                <T as $group_trait<E, Type, KeysTail, CTail>>::$apply_group_method(
                    self, tail,
                );
            }
        }
    };
}

impl_same_as_column_preparation! {
    kind = "mandatory",
    index = MandatorySameAsIndex,
    leaf_trait = PrepareMandatoryColumn,
    has_method = has_mandatory_target,
    prepare_leaf_method = prepare_mandatory_column,
    apply_leaf_method = apply_mandatory_column,
    group_trait = PrepareMandatoryGroup,
    prepare_group_method = prepare_mandatory_group,
    apply_group_method = apply_mandatory_group,
}

impl_same_as_column_preparation! {
    kind = "discretionary",
    index = DiscretionarySameAsIndex,
    leaf_trait = PrepareDiscretionaryColumn,
    has_method = has_discretionary_target,
    prepare_leaf_method = prepare_discretionary_column,
    apply_leaf_method = apply_discretionary_column,
    group_trait = PrepareDiscretionaryGroup,
    prepare_group_method = prepare_discretionary_group,
    apply_group_method = apply_discretionary_group,
}
