//! Implementation module for tuple trait generators.
//!
//! This module contains the logic for generating trait implementations
//! for tuples, replacing the complex macro_rules! patterns with cleaner
//! procedural macros.

use proc_macro2::TokenStream;
use quote::quote;

use crate::tuple_generator::{generate_all_sizes, type_params};

/// Generate DefaultTuple trait implementations for all tuple sizes
pub fn generate_default_tuple() -> TokenStream {
    let impls = generate_all_sizes(|size| {
        if size == 0 {
            quote! {
                impl DefaultTuple for () {
                    fn default_tuple() -> Self {
                        ()
                    }
                }
            }
        } else {
            let type_params = type_params(size);
            let defaults = type_params.iter().map(|_| quote! { Default::default() });

            quote! {
                impl<#(#type_params),*> DefaultTuple for (#(#type_params,)*)
                where
                    #(#type_params: Default),*
                {
                    fn default_tuple() -> Self {
                        (#(#defaults),*)
                    }
                }
            }
        }
    });

    quote! {
        #impls
    }
}

/// Generate OptionTuple and TransposeOptionTuple trait implementations
pub fn generate_option_tuple() -> TokenStream {
    let impls = generate_all_sizes(|size| {
        if size == 0 {
            quote! {
                impl OptionTuple for () {
                    type Output = ();

                    fn into_option(self) -> Self::Output {
                        ()
                    }
                }

                impl TransposeOptionTuple for () {
                    type Transposed = ();

                    fn transpose_option(self) -> Option<Self::Transposed> {
                        Some(())
                    }
                }
            }
        } else {
            let type_params = type_params(size);
            let indices: Vec<_> = (0..size).map(syn::Index::from).collect();

            let option_types: Vec<_> = type_params.iter().map(|t| quote! { Option<#t> }).collect();
            let indices_tokens: Vec<_> = indices.iter().collect();

            // For single-element tuples, trailing comma is needed in value context
            let (into_value, transpose_value) = if size == 1 {
                (
                    quote! { (#(Some(self.#indices_tokens),)*) },
                    quote! { Some((#(self.#indices_tokens?,)*)) },
                )
            } else {
                (
                    quote! { (#(Some(self.#indices_tokens)),*) },
                    quote! { Some((#(self.#indices_tokens?),*)) },
                )
            };

            quote! {
                impl<#(#type_params),*> OptionTuple for (#(#type_params,)*) {
                    type Output = (#(#option_types,)*);

                    fn into_option(self) -> Self::Output {
                        #into_value
                    }
                }

                impl<#(#type_params),*> TransposeOptionTuple for (#(#option_types,)*) {
                    type Transposed = (#(#type_params,)*);

                    fn transpose_option(self) -> Option<Self::Transposed> {
                        #transpose_value
                    }
                }
            }
        }
    });

    quote! {
        #impls
    }
}

/// Generate RefTuple trait implementations
pub fn generate_ref_tuple() -> TokenStream {
    let impls = generate_all_sizes(|size| {
        if size == 0 {
            quote! {
                impl RefTuple for () {
                    type Output<'a> = () where Self: 'a;
                }
            }
        } else {
            let type_params = type_params(size);
            let ref_types: Vec<_> = type_params.iter().map(|t| quote! { &'a #t }).collect();

            quote! {
                impl<#(#type_params),*> RefTuple for (#(#type_params,)*) {
                    type Output<'a> = (#(#ref_types,)*) where Self: 'a;
                }
            }
        }
    });

    quote! {
        #impls
    }
}

/// Generate Columns trait implementations
pub fn generate_columns() -> TokenStream {
    let impls = generate_all_sizes(|size| {
        if size == 0 {
            quote! {
                impl Columns for () {
                    type Types = ();
                    type Tables = ();
                }
            }
        } else {
            let type_params = type_params(size);
            let column_types: Vec<_> = type_params
                .iter()
                .map(|t| {
                    quote! { <#t as TypedColumn>::Type }
                })
                .collect();
            let table_types: Vec<_> = type_params
                .iter()
                .map(|t| {
                    quote! { <#t as diesel::Column>::Table }
                })
                .collect();

            // Projection impl - requires all columns have same table
            let first_type = &type_params[0];
            let projection_bounds: Vec<_> = type_params
                .iter()
                .skip(1)
                .map(|t| {
                    quote! { #t: TypedColumn<Table=<#first_type as diesel::Column>::Table> }
                })
                .collect();

            quote! {
                impl<#(#type_params),*> Columns for (#(#type_params,)*)
                where #(#type_params: TypedColumn),*
                {
                    type Types = (#(#column_types,)*);
                    type Tables = (#(#table_types,)*);
                }

                impl<#(#type_params),*> Projection for (#(#type_params,)*)
                where #first_type: TypedColumn, #(#projection_bounds),*
                {
                    type Table = <#first_type as diesel::Column>::Table;
                }

                impl<#(#type_params),*> HomogeneousColumns for (#(#type_params,)*)
                where #(#type_params: TypedColumn),*
                {
                    type Type = <#first_type as TypedColumn>::Type;
                }
            }
        }
    });

    quote! {
        #impls
    }
}

/// Generate Tables trait implementations
pub fn generate_tables() -> TokenStream {
    let impls = generate_all_sizes(|size| {
        if size == 0 {
            quote! {
                impl Tables for () {
                    type Models = ();
                    type InsertableModels = ();
                }

                impl TableModels for () {
                    type Tables = ();
                }

                impl InsertableTableModels for () {
                    type Tables = ();
                }
            }
        } else {
            let type_params = type_params(size);

            let model_types: Vec<_> = type_params
                .iter()
                .map(|t| {
                    quote! { <#t as TableAddition>::Model }
                })
                .collect();
            let insertable_model_types: Vec<_> = type_params
                .iter()
                .map(|t| {
                    quote! { <#t as TableAddition>::InsertableModel }
                })
                .collect();
            let table_types: Vec<_> = type_params
                .iter()
                .map(|t| {
                    quote! { <#t as HasTable>::Table }
                })
                .collect();
            let primary_key_types: Vec<_> = type_params
                .iter()
                .map(|t| {
                    quote! { <#t as diesel::Table>::PrimaryKey }
                })
                .collect();

            quote! {
                impl<#(#type_params),*> Tables for (#(#type_params,)*)
                where #(#type_params: TableAddition),*
                {
                    type Models = (#(#model_types,)*);
                    type InsertableModels = (#(#insertable_model_types,)*);
                }

                impl<#(#type_params),*> NonCompositePrimaryKeyTables for (#(#type_params,)*)
                where #(#type_params: crate::table_addition::HasPrimaryKey),*
                {
                    type PrimaryKeys = (#(#primary_key_types,)*);
                }

                impl<#(#type_params),*> TableModels for (#(#type_params,)*)
                where #(#type_params: TableModel),*
                {
                    type Tables = (#(#table_types,)*);
                }

                impl<#(#type_params),*> InsertableTableModels for (#(#type_params,)*)
                where #(#type_params: InsertableTableModel),*
                {
                    type Tables = (#(#table_types,)*);
                }
            }
        }
    });

    quote! {
        #impls
    }
}

/// Generate GetColumns and related trait implementations
pub fn generate_get_columns() -> TokenStream {
    let impls = generate_all_sizes(|size| {
        if size == 0 {
            return quote! {};
        }

        let type_params = type_params(size);
        let first_type = &type_params[0];

        let get_calls: Vec<_> = type_params
            .iter()
            .map(|t| {
                quote! {
                    <T as GetColumn<#t>>::get(self)
                }
            })
            .collect();

        // For single-element tuples, trailing comma is needed
        let get_tuple = if size == 1 {
            quote! { (#(#get_calls,)*) }
        } else {
            quote! { (#(#get_calls),*) }
        };

        let may_get_calls: Vec<_> = type_params
            .iter()
            .map(|t| {
                quote! {
                    <T as MayGetColumn<#t>>::maybe_get(self)
                }
            })
            .collect();

        let may_get_tuple = if size == 1 {
            quote! { (#(#may_get_calls,)*) }
        } else {
            quote! { (#(#may_get_calls),*) }
        };

        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();

        let set_individual_calls: Vec<_> = type_params
            .iter()
            .zip(&indices)
            .map(|(t, idx)| {
                quote! {
                    <T as crate::set_column::SetColumn<#t>>::set(self, &values.#idx);
                }
            })
            .collect();

        let set_calls: Vec<_> = type_params
            .iter()
            .map(|t| {
                quote! {
                    <T as SetInsertableTableModelColumn<#t>>::set(self, value);
                }
            })
            .collect();

        let try_set_individual_calls: Vec<_> = type_params
            .iter()
            .zip(&indices)
            .map(|(t, idx)| {
                quote! {
                    <T as crate::set_column::TrySetColumn<#t>>::try_set(self, &values.#idx)?;
                }
            })
            .collect();

        let try_set_calls: Vec<_> = type_params
            .iter()
            .map(|t| {
                quote! {
                    <T as crate::set_column::TrySetColumn<#t>>::try_set(self, value)?;
                }
            })
            .collect();
        let same_type_bounds: Vec<_> = type_params
            .iter()
            .skip(1)
            .map(|t| {
                quote! { #t: TypedColumn<Type=#first_type::Type> }
            })
            .collect();

        quote! {
            impl<T, #(#type_params),*> GetColumns<(#(#type_params,)*)> for T
            where T: GetColumn<#first_type>, #(T: GetColumn<#type_params>),*,
                    #first_type: TypedColumn, #(#type_params: TypedColumn),*
            {
                fn get(&self) -> <<(#(#type_params,)*) as Columns>::Types as crate::RefTuple>::Output<'_> {
                    #get_tuple
                }
            }

            impl<T, #(#type_params),*> MayGetColumns<(#(#type_params,)*)> for T
            where T: MayGetColumn<#first_type>, #(T: MayGetColumn<#type_params>),*,
                    #first_type: TypedColumn, #(#type_params: TypedColumn),*
            {
                fn may_get(&self) -> <<<(#(#type_params,)*) as Columns>::Types as crate::RefTuple>::Output<'_> as OptionTuple>::Output {
                    #may_get_tuple
                }
            }

            impl<T, #(#type_params),*> SetColumns<(#(#type_params,)*)> for T
            where T: crate::set_column::SetColumn<#first_type>, #(T: crate::set_column::SetColumn<#type_params>),*,
                    #first_type: TypedColumn, #(#type_params: TypedColumn),*
            {
                fn set(&mut self, values: <<(#(#type_params,)*) as Columns>::Types as crate::RefTuple>::Output<'_>) {
                    #(#set_individual_calls)*
                }
            }

            impl<T, #(#type_params),*> SetInsertableTableModelHomogeneousColumn<(#(#type_params,)*)> for T
            where
                T: SetInsertableTableModelColumn<#first_type>,
                #(T: SetInsertableTableModelColumn<#type_params>),*,
                #first_type: TypedColumn,
                #(#same_type_bounds),*
            {
                fn set(&mut self, value: &<#first_type as TypedColumn>::Type) {
                    #(#set_calls)*
                }
            }

            impl<T, #(#type_params),*> TrySetColumns<(#(#type_params,)*)> for T
            where
                T: crate::set_column::TrySetColumn<#first_type>,
                #(T: crate::set_column::TrySetColumn<#type_params>),*,
                #first_type: TypedColumn,
                #(#type_params: TypedColumn),*
            {
                fn try_set(&mut self, values: <<(#(#type_params,)*) as Columns>::Types as crate::RefTuple>::Output<'_>) -> anyhow::Result<()> {
                    #(#try_set_individual_calls)*
                    Ok(())
                }
            }

            impl<T, #(#type_params),*> TrySetInsertableTableModelHomogeneousColumn<(#(#type_params,)*)> for T
            where
                T: crate::set_column::TrySetColumn<#first_type>,
                #(T: crate::set_column::TrySetColumn<#type_params>),*,
                #first_type: TypedColumn,
                #(#same_type_bounds),*
            {
                fn try_set(&mut self, value: &<#first_type as TypedColumn>::Type) -> anyhow::Result<()> {
                    #(#try_set_calls)*
                    Ok(())
                }
            }
        }
    });

    quote! {
        #impls
    }
}

/// Generate NestedInsertTuple trait implementations
pub fn generate_nested_insert_tuple() -> TokenStream {
    let impls = generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();

        let model_types: Vec<_> = type_params
            .iter()
            .map(|t| {
                quote! { <<#t as HasTable>::Table as TableAddition>::Model }
            })
            .collect();
        let indices_tokens: Vec<_> = indices.iter().collect();

        quote! {
            impl<Conn, #(#type_params),*> NestedInsertTuple<Conn> for (#(#type_params,)*)
            where
                Conn: LoadConnection,
                #(#type_params: NestedInsert<Conn> + HasTableAddition),*
            {
                type ModelsTuple = (#(#model_types,)*);

                fn nested_insert_tuple(self, conn: &mut Conn) -> anyhow::Result<Self::ModelsTuple> {
                    Ok((#(self.#indices_tokens.nested_insert(conn)?,)*))
                }
            }
        }
    });

    quote! {
        #impls
    }
}

/// Generate NestedInsertOptionTuple trait implementations
pub fn generate_nested_insert_option_tuple() -> TokenStream {
    let impls = generate_all_sizes(|size| {
        if size == 0 {
            return quote! {};
        }

        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();

        let option_model_types: Vec<_> = type_params
            .iter()
            .map(|t| {
                quote! { Option<<<#t as HasTable>::Table as TableAddition>::Model> }
            })
            .collect();
        let indices_tokens: Vec<_> = indices.iter().collect();

        // For single-element tuples, trailing comma is needed in value context
        let result_value = if size == 1 {
            quote! { Ok((#(match self.#indices_tokens {
                Some(builder) => Some(builder.nested_insert(conn)?),
                None => None,
            },)*)) }
        } else {
            quote! { Ok((#(match self.#indices_tokens {
                Some(builder) => Some(builder.nested_insert(conn)?),
                None => None,
            }),*)) }
        };

        quote! {
            impl<Conn, #(#type_params),*> NestedInsertOptionTuple<Conn> for (#(Option<#type_params>,)*)
            where
                Conn: LoadConnection,
                #(#type_params: NestedInsert<Conn> + HasTableAddition),*
            {
                type OptionModelsTuple = (#(#option_model_types,)*);

                fn nested_insert_option_tuple(self, conn: &mut Conn) -> anyhow::Result<Self::OptionModelsTuple> {
                    #result_value
                }
            }
        }
    });

    quote! {
        #impls
    }
}

/// Generate BuildableTables trait implementations
pub fn generate_buildable_tables() -> TokenStream {
    let impls = generate_all_sizes(|size| {
        if size == 0 {
            quote! {
                impl BuildableTables for () {
                    type Builders = ();
                }
            }
        } else {
            let type_params = type_params(size);
            let builder_types: Vec<_> = type_params
                .iter()
                .map(|t| {
                    quote! { TableBuilder<#t> }
                })
                .collect();

            quote! {
                impl<#(#type_params),*> BuildableTables for (#(#type_params,)*)
                where #(#type_params: BuildableTable),*
                {
                    type Builders = (#(#builder_types,)*);
                }
            }
        }
    });

    quote! {
        #impls
    }
}

/// Generate BuildableColumns trait implementations
pub fn generate_buildable_columns() -> TokenStream {
    let impls = generate_all_sizes(|size| {
        if size == 0 {
            quote! {
                impl BuildableColumns for () {
                    type Tables = ();
                }
            }
        } else {
            let type_params = type_params(size);
            let table_types: Vec<_> = type_params
                .iter()
                .map(|t| {
                    quote! { <#t as diesel::Column>::Table }
                })
                .collect();

            quote! {
                impl<#(#type_params),*> BuildableColumns for (#(#type_params,)*)
                where #(#type_params: BuildableColumn),*
                {
                    type Tables = (#(#table_types,)*);
                }
            }
        }
    });

    quote! {
        #impls
    }
}
