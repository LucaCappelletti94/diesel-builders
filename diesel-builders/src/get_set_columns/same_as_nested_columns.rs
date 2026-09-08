//! Traits for fallibly and infallibly setting columns that participate in
//! mandatory or discretionary same-as relationships.
//!
//! The mandatory and discretionary variants are structurally identical, so both
//! trait families are emitted from the `impl_same_as_nested_columns` macro,
//! keyed by their same-as marker, per-column setter, and generated identifiers.

use crate::{
    DiscretionarySameAsIndex, MandatorySameAsIndex, OptionalRef, TrySetDiscretionarySameAsColumn,
    TrySetMandatorySameAsColumn, TypedColumn, columns::NestedColumns,
};

/// Emits a `TrySet…SameAsNestedColumns` / `Set…SameAsNestedColumns` trait
/// family for one same-as flavour.
///
/// The two flavours differ only in their marker trait, per-column setter, and
/// the generated identifiers, so the whole family is produced from this single
/// template.
macro_rules! impl_same_as_nested_columns {
    (
        kind = $kind:literal,
        index = $index:ident,
        column_trait = $col_trait:ident,
        column_method = $col_method:ident,
        try_trait = $try_trait:ident,
        try_method = $try_method:ident,
        set_trait = $set_trait:ident,
        set_method = $set_method:ident $(,)?
    ) => {
        #[doc = concat!(
            "Trait for fallibly setting columns in a ", $kind,
            " same-as relationship."
        )]
        pub trait $try_trait<Type, Error, Keys: NestedColumns, CS: NestedColumns> {
            #[doc = concat!(
                "Attempts to set the value of the specified columns in the ", $kind,
                " same-as relationship.\n\n# Errors\n\nReturns an error if the column values \
                 cannot be set in the ", $kind, " same-as relationship."
            )]
            fn $try_method(
                &mut self,
                value: &impl OptionalRef<Type>,
            ) -> Result<&mut Self, Error>;
        }

        impl<T, Type, Error> $try_trait<Type, Error, (), ()> for T {
            #[inline]
            fn $try_method(
                &mut self,
                _value: &impl OptionalRef<Type>,
            ) -> Result<&mut Self, Error> {
                Ok(self)
            }
        }

        impl<
            Type: Clone,
            T,
            Error,
            Key: $index,
            Column: TypedColumn<Table = Key::ReferencedTable>,
        > $try_trait<Type, Error, (Key,), (Column,)> for T
        where
            Column::ColumnType: From<Type>,
            T: $col_trait<Key, Column>,
            Error: From<<T as $col_trait<Key, Column>>::Error>,
        {
            #[inline]
            fn $try_method(
                &mut self,
                value: &impl OptionalRef<Type>,
            ) -> Result<&mut Self, Error> {
                if let Some(value) = value.as_optional_ref() {
                    self.$col_method(value.clone())?;
                }
                Ok(self)
            }
        }

        impl<
            T,
            Error,
            Type: Clone,
            KeysHead: $index,
            KeysTail: NestedColumns,
            CHead: TypedColumn<Table = KeysHead::ReferencedTable>,
            CTail: NestedColumns,
        > $try_trait<Type, Error, (KeysHead, KeysTail), (CHead, CTail)> for T
        where
            (KeysHead, KeysTail): NestedColumns,
            (CHead, CTail): NestedColumns,
            CHead::ColumnType: From<Type>,
            T: $col_trait<KeysHead, CHead> + $try_trait<Type, Error, KeysTail, CTail>,
            Error: From<<T as $col_trait<KeysHead, CHead>>::Error>,
        {
            #[inline]
            fn $try_method(
                &mut self,
                value: &impl OptionalRef<Type>,
            ) -> Result<&mut Self, Error> {
                if let Some(value) = value.as_optional_ref() {
                    self.$col_method(value.clone())?;
                }
                <T as $try_trait<Type, Error, KeysTail, CTail>>::$try_method(self, value)
            }
        }

        #[doc = concat!(
            "Trait for infallibly setting columns in a ", $kind,
            " same-as relationship."
        )]
        pub trait $set_trait<Type, Keys: NestedColumns, CS: NestedColumns> {
            #[doc = concat!(
                "Sets the value of the specified columns in the ", $kind,
                " same-as relationship."
            )]
            fn $set_method(&mut self, value: &impl OptionalRef<Type>) -> &mut Self;
        }

        impl<T, Type, Keys, CS> $set_trait<Type, Keys, CS> for T
        where
            Keys: NestedColumns,
            CS: NestedColumns,
            T: $try_trait<Type, core::convert::Infallible, Keys, CS>,
        {
            #[inline]
            fn $set_method(&mut self, value: &impl OptionalRef<Type>) -> &mut Self {
                self.$try_method(value).unwrap_or_else(|err| match err {})
            }
        }
    };
}

impl_same_as_nested_columns! {
    kind = "mandatory",
    index = MandatorySameAsIndex,
    column_trait = TrySetMandatorySameAsColumn,
    column_method = try_set_mandatory_same_as_column,
    try_trait = TrySetMandatorySameAsNestedColumns,
    try_method = try_set_mandatory_same_as_nested_columns,
    set_trait = SetMandatorySameAsNestedColumns,
    set_method = set_mandatory_same_as_nested_columns,
}

impl_same_as_nested_columns! {
    kind = "discretionary",
    index = DiscretionarySameAsIndex,
    column_trait = TrySetDiscretionarySameAsColumn,
    column_method = try_set_discretionary_same_as_column,
    try_trait = TrySetDiscretionarySameAsNestedColumns,
    try_method = try_set_discretionary_same_as_nested_columns,
    set_trait = SetDiscretionarySameAsNestedColumns,
    set_method = set_discretionary_same_as_nested_columns,
}
