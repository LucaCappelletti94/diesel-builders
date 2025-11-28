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

                impl<T: diesel::Table> Projection<T> for () {}
            }
        } else {
            let type_params = type_params(size);

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
                impl<#(#type_params),*> NonEmptyProjection for (#(#type_params,)*)
                where #first_type: TypedColumn, #(#projection_bounds),*
                {
                    type Table = <#first_type as diesel::Column>::Table;
                }

                impl<#(#type_params),*> Columns for (#(#type_params,)*)
                where #(#type_params: TypedColumn),*
                {
                    type Types = (#(<#type_params as TypedColumn>::Type,)*);
                    type Tables = (#(<#type_params as diesel::Column>::Table,)*);
                }

                impl<#(#type_params),*> Projection<<#first_type as diesel::Column>::Table> for (#(#type_params,)*)
                where #first_type: TypedColumn, #(#projection_bounds),*
                {
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
        let type_params = type_params(size);

        let maybe_where = if size == 0 { None } else { Some(quote! { where }) };

        quote! {
            impl<#(#type_params),*> Tables for (#(#type_params,)*)
                #maybe_where #(#type_params: TableAddition,)*
            {
                type Models = (#(<#type_params as TableAddition>::Model,)*);
                type InsertableModels = (#(<#type_params as TableAddition>::InsertableModel,)*);
            }
            impl<#(#type_params),*> NonCompositePrimaryKeyTables for (#(#type_params,)*)
                #maybe_where #(#type_params: crate::table_addition::HasPrimaryKey,)*
            {
                type PrimaryKeys = (#(<#type_params as diesel::Table>::PrimaryKey,)*);
            }
            impl<#(#type_params),*> TableModels for (#(#type_params,)*)
                #maybe_where #(#type_params: TableModel,)*
            {
                type Tables = (#(<#type_params as HasTable>::Table,)*);
            }
            impl<#(#type_params),*> InsertableTableModels for (#(#type_params,)*)
                #maybe_where #(#type_params: InsertableTableModel,)*
            {
                type Tables = (#(<#type_params as HasTable>::Table,)*);
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

        let try_set_individual_calls: Vec<_> = type_params
            .iter()
            .zip(&indices)
            .map(|(t, idx)| {
                quote! {
                    <T as crate::set_column::TrySetColumn<#t>>::try_set(self, &values.#idx)?;
                }
            })
            .collect();

        let try_may_set_individual_calls: Vec<_> = type_params
            .iter()
            .zip(&indices)
            .map(|(t, idx)| {
                quote! {
                    if let Some(value) = values.#idx {
                        <T as crate::set_column::TrySetColumn<#t>>::try_set(self, value)?;
                    }
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
                    (#(<T as GetColumn<#type_params>>::get(self),)*)
                }
            }

            impl<T, #(#type_params),*> MayGetColumns<(#(#type_params,)*)> for T
            where T: MayGetColumn<#first_type>, #(T: MayGetColumn<#type_params>),*,
                    #first_type: TypedColumn, #(#type_params: TypedColumn),*
            {
                fn may_get(&self) -> <<<(#(#type_params,)*) as Columns>::Types as crate::RefTuple>::Output<'_> as OptionTuple>::Output {
                    (#(<T as MayGetColumn<#type_params>>::may_get(self),)*)
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

            impl<T, #(#type_params),*> SetHomogeneousColumn<(#(#type_params,)*)> for T
            where
                T: SetColumn<#first_type>,
                #(T: SetColumn<#type_params>),*,
                #first_type: TypedColumn,
                #(#same_type_bounds),*
            {
                fn set(&mut self, value: &<#first_type as TypedColumn>::Type) {
                    #(<T as SetColumn<#type_params>>::set(self, value);)*
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

            impl<T, #(#type_params),*> TryMaySetColumns<(#(#type_params,)*)> for T
            where
                T: crate::set_column::TrySetColumn<#first_type>,
                #(T: crate::set_column::TrySetColumn<#type_params>),*,
                #first_type: TypedColumn,
                #(#type_params: TypedColumn),*
            {
                fn try_may_set(&mut self, values: <<<(#(#type_params,)*) as Columns>::Types as crate::RefTuple>::Output<'_> as OptionTuple>::Output) -> anyhow::Result<()> {
                    #(#try_may_set_individual_calls)*
                    Ok(())
                }
            }

            impl<T, #(#type_params),*> TrySetHomogeneousColumn<(#(#type_params,)*)> for T
            where
                T: crate::set_column::TrySetColumn<#first_type>,
                #(T: crate::set_column::TrySetColumn<#type_params>),*,
                #first_type: TypedColumn,
                #(#same_type_bounds),*
            {
                fn try_set(&mut self, value: &<#first_type as TypedColumn>::Type) -> anyhow::Result<()> {
                    #(<T as crate::set_column::TrySetColumn<#type_params>>::try_set(self, value)?;)*
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
        let type_params = type_params(size);

        let where_statement = if size == 0 {
            quote! {}
        } else {
            quote! { where #(#type_params: BuildableTable),* }
        };

        quote! {
            impl<#(#type_params),*> BuildableTables for (#(#type_params,)*)
            #where_statement
            {
                type Builders = (#(TableBuilder<#type_params>,)*);
                type BuilderBundles = (#(TableBuilderBundle<#type_params>,)*);
                type CompletedBuilderBundles = (#(CompletedTableBuilderBundle<#type_params>,)*);
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
        let type_params = type_params(size);

        quote! {
            impl<#(#type_params),*> BuildableColumns for (#(#type_params,)*)
            where
                Self::Tables: BuildableTables,
                #(#type_params: BuildableColumn),*
            {}
        }
    });

    quote! {
        #impls
    }
}

/// Generate NonCompositePrimaryKeyTableModels and MayGetPrimaryKeys trait
/// implementations
pub fn generate_table_model() -> TokenStream {
    let impls = generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();

        let get_pk_calls: Vec<_> = indices
            .iter()
            .map(|idx| {
                quote! {
                    GetColumn::get(&self.#idx)
                }
            })
            .collect();

        // For may_get_primary_keys, build extractors for each optional model
        let pk_extractors: Vec<_> = type_params
            .iter()
            .zip(&indices)
            .map(|(t, idx)| {
                quote! {
                    optional_self.#idx.as_ref().map(|model: &#t| GetColumn::get(model))
                }
            })
            .collect();

        // Build the PrimaryKeys type for the where clause
        let primary_key_types: Vec<_> = type_params
            .iter()
            .map(|t| {
                quote! { <<#t as HasTable>::Table as diesel::Table>::PrimaryKey }
            })
            .collect();

        quote! {
            impl<#(#type_params),*> NonCompositePrimaryKeyTableModels for (#(#type_params,)*)
            where
                #(#type_params: NonCompositePrimaryKeyTableModel,)*
                Self::Tables: NonCompositePrimaryKeyTables<
                    PrimaryKeys = (#(#primary_key_types,)*),
                >,
            {
                fn get_primary_keys(&self) -> <<<Self::Tables as NonCompositePrimaryKeyTables>::PrimaryKeys as Columns>::Types as crate::RefTuple>::Output<'_> {
                    (#(#get_pk_calls,)*)
                }

                fn may_get_primary_keys(optional_self: &<Self as OptionTuple>::Output) -> <<<<Self::Tables as NonCompositePrimaryKeyTables>::PrimaryKeys as Columns>::Types as crate::RefTuple>::Output<'_> as OptionTuple>::Output {
                    (#(#pk_extractors,)*)
                }
            }
        }
    });

    quote! {
        #impls
    }
}

/// Generate BuilderBundles trait implementations
pub fn generate_builder_bundles() -> TokenStream {
    let impls = generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();

        // Generate try_from calls for each element
        let try_from_calls: Vec<_> = indices
            .iter()
            .map(|idx| {
                quote! {
                    CompletedTableBuilderBundle::try_from(self.#idx)?
                }
            })
            .collect();

        let where_statement = if size == 0 {
            quote! {}
        } else {
            quote! { where #(#type_params: TableBundle),* }
        };

        quote! {
            impl<#(#type_params),*> BuilderBundles for (#(TableBuilderBundle<#type_params>,)*)
            #where_statement
            {
                type CompletedBundles = (#(CompletedTableBuilderBundle<#type_params>,)*);

                fn try_complete(self) -> anyhow::Result<Self::CompletedBundles> {
                    Ok((#(#try_from_calls,)*))
                }
            }
        }
    });

    quote! {
        #impls
    }
}

/// Generate AncestorsOf trait implementations
pub fn generate_ancestors_of() -> TokenStream {
    let impls = generate_all_sizes(|size| {
        let type_params = type_params(size);

        // Generate where clauses for T: DescendantOf<A1>, T: DescendantOf<A2>, etc.
        let descendant_of_bounds: Vec<_> = type_params
            .iter()
            .map(|t| {
                quote! { T: DescendantOf<#t> }
            })
            .collect();

        quote! {
            impl<T, #(#type_params),*> AncestorsOf<T> for (#(#type_params,)*)
            where
                T: Descendant<Ancestors = Self>,
                #(#type_params: AncestorOfIndex<T>,)*
                #(#descendant_of_bounds,)*
            {
            }
        }
    });

    quote! {
        #impls
    }
}

/// Generate HorizontalSameAsKeys trait implementations
pub fn generate_horizontal_same_as_keys() -> TokenStream {
    let impls = generate_all_sizes(|size| {
        let type_params = type_params(size);

        quote! {
            impl<T, #(#type_params),*> HorizontalSameAsKeys<T> for (#(#type_params,)*)
            where
                T: diesel::Table,
                #(#type_params: HorizontalSameAsKey<Table = T>,)*
            {
                type ReferencedTables = (#(<#type_params as SingletonForeignKey>::ReferencedTable,)*);
            }
        }
    });

    quote! {
        #impls
    }
}
