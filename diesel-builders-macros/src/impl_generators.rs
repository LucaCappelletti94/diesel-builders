//! Implementation module for tuple trait generators.
//!
//! This module contains the logic for generating trait implementations
//! for tuples, replacing the complex macro_rules! patterns with cleaner
//! procedural macros.

use proc_macro2::TokenStream;
use quote::quote;

use crate::tuple_generator::{
    generate_all_sizes, generate_all_sizes_non_empty, type_params, type_params_with_prefix,
};

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
/// Generate NonEmptyProjection trait implementations
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

/// Generate GetColumns trait implementations
pub fn generate_get_columns_trait() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let where_statement = (size > 0).then(|| quote! { where #(T: GetColumn<#type_params>,)* });

        quote! {
            impl<T, #(#type_params: TypedColumn),*> GetColumns<(#(#type_params,)*)> for T
            #where_statement
            {
                #[inline]
                fn get_columns(&self) -> <<(#(#type_params,)*) as Typed>::Type as TupleRef>::Ref<'_> {
                    (#(<T as GetColumn<#type_params>>::get_column(self),)*)
                }
            }
        }
    })
}

/// Generate `TupleGetColumns` trait implementations
pub fn generate_tuple_get_columns_trait() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let column_type_params = type_params_with_prefix(size, "C");
        let indices = (0..size).map(syn::Index::from);

        quote! {
            impl<#(#type_params: GetColumn<#column_type_params>,)* #(#column_type_params: TypedColumn,)*> TupleGetColumns<(#(#column_type_params,)*)> for (#(#type_params,)*)
            {
                #[inline]
                fn tuple_get_columns(&self) -> <<(#(#column_type_params,)*) as Typed>::Type as TupleRef>::Ref<'_> {
                    (#(<#type_params as GetColumn<#column_type_params>>::get_column(&self.#indices),)*)
                }
            }
        }
    })
}

/// Generate MayGetColumns trait implementations
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
                fn may_get_columns(&self) -> <<<(#(#type_params,)*) as Typed>::Type as TupleRef>::Ref<'_> as IntoTupleOption>::IntoOptions {
                    (#(<T as MayGetColumn<#type_params>>::may_get_column(self),)*)
                }
            }
        }
    })
}

/// Generate `TupleMayGetColumns` trait implementations
pub fn generate_tuple_may_get_columns_trait() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let column_type_params = type_params_with_prefix(size, "C");
        let indices = (0..size).map(syn::Index::from);

        quote! {
            impl<#(#type_params: MayGetColumn<#column_type_params>,)* #(#column_type_params: TypedColumn,)*> TupleMayGetColumns<(#(#column_type_params,)*)> for (#(#type_params,)*)
            {
                #[inline]
                fn tuple_may_get_columns(&self) -> <<<(#(#column_type_params,)*) as Typed>::Type as TupleRef>::Ref<'_> as IntoTupleOption>::IntoOptions {
                    (#(<#type_params as MayGetColumn<#column_type_params>>::may_get_column(&self.#indices),)*)
                }
            }
        }
    })
}

/// Generate SetColumns trait implementations
pub fn generate_set_columns_trait() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();
        let set_individual_calls: Vec<_> = type_params
            .iter()
            .zip(&indices)
            .map(|(t, idx)| {
                quote! {
                    <T as crate::set_column::SetColumn<#t>>::set_column(self, values.#idx.clone());
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
                fn set_columns(&mut self, values: <<(#(#type_params,)*) as Typed>::Type as TupleRef>::Ref<'_>) -> &mut Self {
                    #(#set_individual_calls)*
                    self
                }
            }
        }
    })
}

/// Generate MaySetColumns trait implementations
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
                fn may_set_columns(&mut self, values: <<<(#(#type_params,)*) as Typed>::Type as TupleRef>::Ref<'_> as IntoTupleOption>::IntoOptions) -> &mut Self {
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
                fn try_set_columns(&mut self, values: <<(#(#type_params,)*) as Typed>::Type as TupleRef>::Ref<'_>) -> Result<&mut Self, Error> {
                    #(<T as TrySetColumn<#type_params>>::try_set_column(self, values.#indices.clone())?;)*
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
            impl<Error, T, #(#type_params: Typed<Type: TupleRef>,)*> TrySetColumnsCollection<Error, (#(#type_params,)*)> for T
            #where_statement
            {
                #[inline]
                fn try_set_columns_collection(&mut self, values: <<(#(#type_params,)*) as Typed>::Type as TupleRefMap>::RefMap<'_>) -> Result<&mut Self, Error> {
                    #(<T as TrySetColumns<Error, #type_params>>::try_set_columns(self, values.#indices)?;)*
                    Ok(self)
                }
            }
        }
    })
}

/// Generate TryMaySetColumns trait implementations
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
                        <T as TrySetColumn<#t>>::try_set_column(self, value.clone())?;
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
                fn try_may_set_columns(&mut self, values: <<<(#(#type_params,)*) as Typed>::Type as TupleRef>::Ref<'_> as IntoTupleOption>::IntoOptions) -> Result<&mut Self, Error> {
                    #(#try_may_set_individual_calls)*
                    Ok(self)
                }
            }
        }
    })
}

/// Generate InsertTuple trait implementations
pub fn generate_insert_tuple() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();

        let model_types = type_params
            .iter()
            .map(|t| quote! { <<#t as HasTable>::Table as TableAddition>::Model });

        quote! {
            impl<Error, Conn, #(#type_params),*> InsertTuple<Error, Conn> for (#(#type_params,)*)
            where
                Conn: diesel::connection::LoadConnection,
                #(#type_params: crate::RecursiveBuilderInsert<Error, Conn> + crate::HasTableAddition,)*
            {
                type ModelsTuple = (#(#model_types,)*);

                #[inline]
                fn insert_tuple(self, conn: &mut Conn) -> crate::BuilderResult<Self::ModelsTuple, Error> {
                    Ok((#(self.#indices.recursive_insert(conn)?,)*))
                }
            }
        }
    })
}

/// Generate InsertOptionTuple trait implementations
pub fn generate_insert_option_tuple() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();

        let option_model_types = type_params
            .iter()
            .map(|t| quote! { Option<<<#t as HasTable>::Table as TableAddition>::Model> });

        quote! {
            impl<Error, Conn, #(#type_params,)*> InsertOptionTuple<Error, Conn> for (#(Option<#type_params>,)*)
            where
                Conn: diesel::connection::LoadConnection,
                #(#type_params: crate::RecursiveBuilderInsert<Error, Conn> + crate::HasTableAddition,)*
            {
                type OptionModelsTuple = (#(#option_model_types,)*);

                fn insert_option_tuple(self, conn: &mut Conn) -> crate::BuilderResult<Self::OptionModelsTuple, Error> {
                    Ok((#(match self.#indices {
                        Some(builder) => Some(builder.recursive_insert(conn)?),
                        None => None,
                    },)*))
                }
            }
        }
    })
}

/// Generate BuildableTables trait implementations
pub fn generate_buildable_tables() -> TokenStream {
    generate_all_sizes(|size| {
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

/// Generate BundlableTables trait implementations
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

/// Generate AncestorsOf trait implementations
pub fn generate_ancestors_of() -> TokenStream {
    generate_all_sizes(|size| {
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

/// Generate HorizontalSameAsKeys trait implementations
pub fn generate_horizontal_same_as_keys() -> TokenStream {
    generate_all_sizes(|size| {
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
    // Generate implementations for tuples of size 1-32
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
    // Generate implementations for tuples of size 1-32
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
    generate_all_sizes_non_empty(|size| {
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
    generate_all_sizes_non_empty(|size| {
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
