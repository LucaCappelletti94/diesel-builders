//! Implementation module for tuple trait generators.
//!
//! This module contains the logic for generating trait implementations
//! for tuples, replacing the complex `macro_rules`! patterns with cleaner
//! procedural macros.

use proc_macro2::TokenStream;
use quote::quote;

use crate::tuple_generator::{
    generate_all_sizes, generate_all_sizes_non_empty, generate_all_sizes_non_empty_with_max,
    generate_all_sizes_with_max, type_params, type_params_with_prefix,
};

/// Maximum number of columns in composite indexes or foreign keys.
/// Limited to 8 because composite keys with more columns are extremely rare
/// and indicate poor database design. Also helps reduce compile times.
pub const COMPOSITE_KEY_MAX_SIZE: usize = 8;

/// Maximum size for table hierarchies (`BuildableTables` and `AncestorsOf`).
/// These MUST use the same size as they are tightly coupled in the inheritance
/// implementation. Limited to 8 because deep hierarchies cause performance issues.
pub const TABLE_HIERARCHY_MAX_SIZE: usize = 32;

/// Maximum number of horizontal same-as keys (triangular relationships).
/// Limited to 8 to prevent excessive complexity in horizontal relationship chains.
pub const HORIZONTAL_SAME_AS_KEYS_MAX_SIZE: usize = 16;

/// Generate Columns trait implementations
pub fn generate_columns() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);

        quote! {
            impl<#(#type_params: Typed),*> Typed for (#(#type_params,)*)
            {
                type Type = (#(<#type_params as Typed>::Type,)*);
            }
            impl<#(#type_params: crate::TypedColumn),*> Columns for (#(#type_params,)*)
            {
                type Tables = (#(<#type_params as diesel::Column>::Table,)*);
            }
        }
    })
}
/// Generate `NonEmptyProjection` trait implementations
pub fn generate_non_empty_projection() -> TokenStream {
    generate_all_sizes_non_empty(|size| {
        let type_params = type_params(size);
        let first_param = &type_params[0];

        quote! {
            impl<#(#type_params: crate::TypedColumn),*> NonEmptyProjection for (#(#type_params,)*)
            {
                type Table = <#first_param as diesel::Column>::Table;
            }
        }
    })
}

/// Generate Tables trait implementations
pub fn generate_tables() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        quote! {
            impl<#(#type_params: TableAddition),*> Tables for (#(#type_params,)*)
            {
                type Models = (#(<#type_params as TableAddition>::Model,)*);
                type InsertableModels = (#(<#type_params as TableAddition>::InsertableModel,)*);
                type PrimaryKeyColumnsCollection = (#(<#type_params as TableAddition>::PrimaryKeyColumns,)*);
            }
            impl<#(#type_params: HasTable<Table: TableAddition>),*> HasTables for (#(#type_params,)*)
            {
                type Tables = (#(<#type_params as HasTable>::Table,)*);
            }
        }
    })
}

/// Generate `GetColumns` trait implementations
pub fn generate_get_columns_trait() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let where_statement = (size > 0).then(|| quote! { where #(T: GetColumn<#type_params>,)* });

        quote! {
            impl<T, #(#type_params: TypedColumn),*> GetColumns<(#(#type_params,)*)> for T
            #where_statement
            {
                #[inline]
                fn get_columns_ref(&self) -> <<(#(#type_params,)*) as Typed>::Type as TupleRef>::Ref<'_> {
                    (#(<T as GetColumn<#type_params>>::get_column_ref(self),)*)
                }
                #[inline]
                fn get_columns(&self) -> <(#(#type_params,)*) as Typed>::Type {
                    (#(<T as GetColumn<#type_params>>::get_column(self),)*)
                }
            }
        }
    })
}

/// Generate `MayGetColumns` trait implementations
pub fn generate_may_get_columns_trait() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let where_statement =
            (size > 0).then(|| quote! { where #(T: MayGetColumn<#type_params>,)* });

        quote! {
            impl<T, #(#type_params: TypedColumn),*> MayGetColumns<(#(#type_params,)*)> for T
            #where_statement
            {
                #[inline]
                fn may_get_columns(
                    &self,
                ) -> <<(#(#type_params,)*) as Typed>::Type as IntoTupleOption>::IntoOptions {
                    (#(<T as MayGetColumn<#type_params>>::may_get_column(self),)*)
                }
            }
        }
    })
}

/// Generate `SetColumns` trait implementations
pub fn generate_set_columns_trait() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();
        let set_individual_calls: Vec<_> = type_params
            .iter()
            .zip(&indices)
            .map(|(t, idx)| {
                quote! {
                    <T as crate::set_column::SetColumn<#t>>::set_column(self, values.#idx);
                }
            })
            .collect();
        let where_statement =
            (size > 0).then(|| quote! { where #(T: crate::set_column::SetColumn<#type_params>,)* });

        quote! {
            impl<T, #(#type_params: TypedColumn),*> SetColumns<(#(#type_params,)*)> for T
            #where_statement
            {
                #[inline]
                fn set_columns(&mut self, values: <(#(#type_params,)*) as Typed>::Type) -> &mut Self {
                    #(#set_individual_calls)*
                    self
                }
            }
        }
    })
}

/// Generate `MaySetColumns` trait implementations
pub fn generate_may_set_columns_trait() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();
        let may_set_individual_calls: Vec<_> = type_params
            .iter()
            .zip(&indices)
            .map(|(t, idx)| {
                quote! {
                    <T as crate::set_column::MaySetColumn<#t>>::may_set_column(self, values.#idx);
                }
            })
            .collect();
        let where_statement = (size > 0)
            .then(|| quote! { where #(T: crate::set_column::MaySetColumn<#type_params>,)* });

        quote! {
            impl<T, #(#type_params: TypedColumn),*> MaySetColumns<(#(#type_params,)*)> for T
            #where_statement
            {
                #[inline]
                fn may_set_columns(&mut self, values: <<(#(#type_params,)*) as Typed>::Type as IntoTupleOption>::IntoOptions) -> &mut Self {
                    #(#may_set_individual_calls)*
                    self
                }
            }
        }
    })
}

/// Generate `TrySetColumns` trait implementations
pub fn generate_try_set_columns_trait() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();
        let where_statement = (size > 0).then(|| {
            quote! {
                where
                    #(T: TrySetColumn<#type_params>,)*
                    #(Error: From<<T as TrySetColumn<#type_params>>::Error>,)*
            }
        });

        quote! {
            impl<Error, T, #(#type_params: Typed,)*> TrySetColumns<Error, (#(#type_params,)*)> for T
            #where_statement
            {
                #[inline]
                fn try_set_columns(&mut self, values: <(#(#type_params,)*) as Typed>::Type) -> Result<&mut Self, Error> {
                    #(<T as TrySetColumn<#type_params>>::try_set_column(self, values.#indices)?;)*
                    Ok(self)
                }
            }
        }
    })
}

/// Generate `TrySetColumnsCollections` trait implementations
pub fn generate_try_set_columns_collections_trait() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();
        let where_statement = (size > 0).then(|| {
            quote! {
                where
                    #(T: TrySetColumns<Error, #type_params>,)*
            }
        });

        quote! {
            impl<Error, T, #(#type_params: Typed,)*> TrySetColumnsCollection<Error, (#(#type_params,)*)> for T
            #where_statement
            {
                #[inline]
                fn try_set_columns_collection(&mut self, values: <(#(#type_params,)*) as Typed>::Type) -> Result<&mut Self, Error> {
                    #(<T as TrySetColumns<Error, #type_params>>::try_set_columns(self, values.#indices)?;)*
                    Ok(self)
                }
            }
        }
    })
}

/// Generate `TryMaySetColumns` trait implementations
pub fn generate_try_may_set_columns_trait() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();
        let try_may_set_individual_calls: Vec<_> = type_params
            .iter()
            .zip(&indices)
            .map(|(t, idx)| {
                quote! {
                    if let Some(value) = values.#idx {
                        <T as TrySetColumn<#t>>::try_set_column(self, value)?;
                    }
                }
            })
            .collect();
        let where_statement = (size > 0).then(|| quote! {
            where
                #(T: TrySetColumn<#type_params>,)*
                #(Error: From<<T as TrySetColumn<#type_params>>::Error>,)*
                #(<<<Self as HasTable>::Table as TableAddition>::InsertableModel as InsertableTableModel>::Error: From<<T as TrySetColumn<#type_params>>::Error>,)*
        });

        quote! {
            impl<Error, T: HasTableAddition, #(#type_params: TypedColumn),*> TryMaySetColumns<Error, (#(#type_params,)*)> for T
            #where_statement
            {
                #[inline]
                fn try_may_set_columns(&mut self, values: <<(#(#type_params,)*) as Typed>::Type as IntoTupleOption>::IntoOptions) -> Result<&mut Self, Error> {
                    #(#try_may_set_individual_calls)*
                    Ok(self)
                }
            }
        }
    })
}

/// Generate `BuildableTables` trait implementations
pub fn generate_buildable_tables() -> TokenStream {
    let max_size = TABLE_HIERARCHY_MAX_SIZE.min(crate::tuple_generator::MAX_TUPLE_SIZE);
    generate_all_sizes_with_max(max_size, |size| {
        let type_params = type_params(size);
        let where_statement =
            (size > 0).then(|| quote! { where #(#type_params: crate::BuildableTable),* });

        quote! {
            impl<#(#type_params),*> crate::BuildableTables for (#(#type_params,)*)
            #where_statement
            {
                type Builders = (#(crate::TableBuilder<#type_params>,)*);
                type OptionalBuilders = (#(Option<crate::TableBuilder<#type_params>>,)*);
            }
        }
    })
}

/// Generate `BundlableTables` trait implementations
pub fn generate_bundlable_tables() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);

        quote! {
            impl<#(#type_params: crate::BundlableTable),*> BundlableTables for (#(#type_params,)*)
            {
                type BuilderBundles = (#(TableBuilderBundle<#type_params>,)*);
                type CompletedBuilderBundles = (#(CompletedTableBuilderBundle<#type_params>,)*);
            }
        }
    })
}

/// Generate `AncestorsOf` trait implementations
pub fn generate_ancestors_of() -> TokenStream {
    let max_size = TABLE_HIERARCHY_MAX_SIZE.min(crate::tuple_generator::MAX_TUPLE_SIZE);
    generate_all_sizes_with_max(max_size, |size| {
        let type_params = type_params(size);

        // Generate where clauses for T: DescendantOf<A1>, T: DescendantOf<A2>, etc.
        let descendant_of_bounds = type_params.iter().map(|t| quote! { T: DescendantOf<#t> });

        // Generate TableNotEqual bounds: T: TableNotEqual<A1>, T: TableNotEqual<A2>, etc.
        let table_not_equal_bounds = type_params
            .iter()
            .map(|t| quote! { T: diesel::query_source::TableNotEqual<#t> });

        quote! {
            impl<T, #(#type_params),*> AncestorsOf<T> for (#(#type_params,)*)
            where
                T: Descendant<Ancestors = Self>,
                #(#type_params: AncestorOfIndex<T>,)*
                #(#descendant_of_bounds,)*
                #(#table_not_equal_bounds,)*
            {
            }
        }
    })
}

/// Generate `HorizontalSameAsKeys` trait implementations
pub fn generate_horizontal_same_as_keys() -> TokenStream {
    let max_size = HORIZONTAL_SAME_AS_KEYS_MAX_SIZE.min(crate::tuple_generator::MAX_TUPLE_SIZE);
    generate_all_sizes_with_max(max_size, |size| {
        let type_params = type_params(size);
        let additional_requirements = (size > 0).then(|| quote! {+ HasPrimaryKeyColumn });

        quote! {
            impl<T, #(#type_params),*> HorizontalSameAsKeys<T> for (#(#type_params,)*)
            where
                T: crate::TableAddition #additional_requirements,
                #(#type_params: HorizontalSameAsKey<Table = T>,)*
            {
                type ReferencedTables = (#(<#type_params as SingletonForeignKey>::ReferencedTable,)*);
                type HostColumnsMatrix = (#(<#type_params as HorizontalSameAsKey>::HostColumns,)*);
                type ForeignColumnsMatrix = (#(<#type_params as HorizontalSameAsKey>::ForeignColumns,)*);
            }
        }
    })
}

/// Generate `TryMaySetDiscretionarySameAsColumns` trait implementations for all tuple sizes.
pub fn generate_try_may_set_discretionary_same_as_columns() -> TokenStream {
    generate_all_sizes_non_empty(|size| {
        let keys = type_params_with_prefix(size, "K");
        let column_types = type_params_with_prefix(size, "C");

        // Generate the try_set_discretionary_same_as_column calls
        let discretionary_calls = keys.iter().zip(column_types.iter()).map(|(key, column_type)| {
            quote! {
                <Self as TryMaySetDiscretionarySameAsColumn<#key, #column_type>>::try_may_set_discretionary_same_as_column(self, value.clone())?;
            }
        });

        // Generate where clauses for TryMaySetDiscretionarySameAsColumn
        let discretionary_trait_bounds = keys.iter().zip(column_types.iter()).map(|(key, column_type)| {
            quote! {
                Self: TryMaySetDiscretionarySameAsColumn<#key, #column_type>,
                Error: From<<Self as TryMaySetDiscretionarySameAsColumn<#key, #column_type>>::Error>
            }
        });

        quote! {
            impl<
                Type: Clone + 'static,
                Error,
                T,
                #(#keys: DiscretionarySameAsIndex,)*
                #(#column_types: crate::TypedColumn<Table=#keys::ReferencedTable, Type=Type>,)*
            > TryMaySetDiscretionarySameAsColumns<
                Type,
                Error,
                (#(#keys,)*),
                (#(#column_types,)*)
            > for T
            where
                #(#discretionary_trait_bounds,)*
            {
                #[inline]
                fn try_may_set_discretionary_same_as_columns(
                    &mut self,
                    value: &Type
                ) -> Result<&mut Self, Error> {
                    #(#discretionary_calls)*
                    Ok(self)
                }
            }
        }
    })
}

/// Generate `TrySetMandatorySameAsColumns` trait implementations for all tuple sizes.
pub fn generate_try_set_mandatory_same_as_columns() -> TokenStream {
    generate_all_sizes_non_empty(|size| {
        let keys = type_params_with_prefix(size, "K");
        let column_types = type_params_with_prefix(size, "C");

        // Generate the try_set_mandatory_same_as_column calls
        let mandatory_calls = keys.iter().zip(column_types.iter()).map(|(key, column_type)| {
            quote! {
                <Self as TrySetMandatorySameAsColumn<#key, #column_type>>::try_set_mandatory_same_as_column(self, value.clone())?;
            }
        });

        // Generate where clauses for TrySetMandatorySameAsColumn
        let mandatory_trait_bounds = keys.iter().zip(column_types.iter()).map(
            |(key, column_type)| {
                quote! {
                    Self: TrySetMandatorySameAsColumn<#key, #column_type>,
                    Error: From<<Self as TrySetMandatorySameAsColumn<#key, #column_type>>::Error>
                }
            },
        );

        quote! {
            impl<
                Type: Clone + 'static,
                Error,
                #(#keys: MandatorySameAsIndex,)*
                #(#column_types: crate::TypedColumn<Table=#keys::ReferencedTable, Type=Type>,)*
                T,
            > TrySetMandatorySameAsColumns<
                Type,
                Error,
                (#(#keys,)*),
                (#(#column_types,)*)
            > for T
            where
                #(#mandatory_trait_bounds,)*
            {
                #[inline]
                fn try_set_mandatory_same_as_columns(
                    &mut self,
                    value: &Type
                ) -> Result<&mut Self, Error> {
                    #(#mandatory_calls)*
                    Ok(self)
                }
            }
        }
    })
}

/// Generate `TableIndex` trait marker implementations for all tuple sizes.
pub fn generate_table_index() -> proc_macro2::TokenStream {
    let max_size = COMPOSITE_KEY_MAX_SIZE.min(crate::tuple_generator::MAX_TUPLE_SIZE);
    generate_all_sizes_non_empty_with_max(max_size, |size| {
        let type_params = type_params(size);

        // Generate index constants (U0, U1, U2, ...)
        let indices: Vec<_> = (0..size)
            .map(|i| {
                let ident = quote::format_ident!("U{}", i);
                quote! { typenum::#ident }
            })
            .collect();

        // Generate IndexedColumn bounds for each column
        let indexed_column_bounds = type_params.iter().zip(indices.iter()).map(|(param, idx)| {
            quote! {
                #param: IndexedColumn<#idx, (#(#type_params,)*), Table=Self::Table>,
                (#(#type_params,)*): TupleIndex<#idx, Element = #param>
            }
        });

        quote! {
            impl<#(#type_params: TypedColumn,)*> TableIndex for (#(#type_params,)*)
            where
                (#(#type_params,)*): NonEmptyProjection,
                #(#indexed_column_bounds,)*
            {
            }
        }
    })
}

/// Generate `ForeignKey` trait marker implementations for all tuple sizes.
pub fn generate_foreign_key() -> proc_macro2::TokenStream {
    let max_size = COMPOSITE_KEY_MAX_SIZE.min(crate::tuple_generator::MAX_TUPLE_SIZE);
    generate_all_sizes_non_empty_with_max(max_size, |size| {
        let host_params = type_params(size);
        let ref_params = type_params_with_prefix(size, "R");
        // Generate index constants (U0, U1, U2, ...)
        let indices: Vec<_> = (0..size)
            .map(|i| {
                let ident = quote::format_ident!("U{}", i);
                quote! { typenum::#ident }
            })
            .collect();
        // Generate HostColumn bounds for each host parameter with type equality
        let bounds = host_params
            .iter()
            .zip(ref_params.iter())
            .zip(indices.iter())
            .map(|((host_param, ref_param), idx)| {
                quote! {
                    #host_param: HostColumn<
                        #idx,
                        (#(#host_params,)*),
                        (#(#ref_params,)*),
                        Type = <#ref_param as crate::Typed>::Type,
                        Table = Self::Table
                    >,
                    #ref_param: IndexedColumn<
                        #idx,
                        (#(#ref_params,)*),
                        Table = <(#(#ref_params,)*) as NonEmptyProjection>::Table
                    >,
                    (#(#host_params,)*): TupleIndex<#idx, Element = #host_param>,
                    (#(#ref_params,)*): TupleIndex<#idx, Element = #ref_param>
                }
            });

        quote! {
            impl<#(#host_params: TypedColumn,)* #(#ref_params: TypedColumn,)*> ForeignKey<(#(#ref_params,)*)> for (#(#host_params,)*)
            where
                (#(#host_params,)*): NonEmptyProjection,
                (#(#ref_params,)*): NonEmptyProjection,
                #(#bounds,)*
            {
            }
        }
    })
}
