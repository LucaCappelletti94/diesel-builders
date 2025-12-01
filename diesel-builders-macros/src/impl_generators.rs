//! Implementation module for tuple trait generators.
//!
//! This module contains the logic for generating trait implementations
//! for tuples, replacing the complex macro_rules! patterns with cleaner
//! procedural macros.

use proc_macro2::TokenStream;
use quote::quote;

use crate::tuple_generator::{generate_all_sizes, type_params, MAX_TUPLE_SIZE};

/// Generate DefaultTuple trait implementations for all tuple sizes
pub fn generate_default_tuple() -> TokenStream {
    let impls = generate_all_sizes(|size| {
        let type_params = type_params(size);
        let defaults = type_params.iter().map(|_| quote! { Default::default() });

        quote! {
            impl<#(#type_params: Default),*> DefaultTuple for (#(#type_params,)*)
            {
                #[inline]
                fn default_tuple() -> Self {
                    (#(#defaults),*)
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
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();

        let indices_tokens: Vec<_> = indices.iter().collect();

        quote! {
            impl<#(#type_params: Clone + core::fmt::Debug),*> OptionTuple for (#(#type_params,)*) {
                type Output = (#(Option<#type_params>,)*);

                #[inline]
                fn into_option(self) -> Self::Output {
                    (#(Some(self.#indices_tokens),)*)
                }
            }

            impl<#(#type_params: Clone + core::fmt::Debug),*> TransposeOptionTuple for (#(Option<#type_params>,)*) {
                type Transposed = (#(#type_params,)*);

                #[inline]
                fn transpose_option(self) -> Option<Self::Transposed> {
                    Some((#(self.#indices_tokens?,)*))
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
        let type_params = type_params(size);
        quote! {
            impl<#(#type_params: Clone + core::fmt::Debug,)*> RefTuple for (#(#type_params,)*) {
                type Output<'a> = (#(&'a #type_params,)*) where Self: 'a;
            }
        }
    });

    quote! {
        #impls
    }
}

/// Generate ClonableTuple trait implementations for all tuple sizes
pub fn generate_clonable_tuple() -> TokenStream {
    let impls = generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();
        let clones = indices.iter().map(|idx| quote! { self.#idx.clone() });

        quote! {
            impl<#(#type_params: Clone),*> ClonableTuple for (#(#type_params,)*)
            {
                #[inline]
                fn clone_tuple(&self) -> Self {
                    (#(#clones,)*)
                }
            }
        }
    });

    quote! {
        #impls
    }
}

/// Generate DebuggableTuple trait implementations for all tuple sizes
pub fn generate_debuggable_tuple() -> TokenStream {
    let impls = generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();
        let format_str = format!("({})", vec!["{:?}"; size].join(", "));
        let debug_refs = indices.iter().map(|idx| quote! { &self.#idx });

        quote! {
            impl<#(#type_params: std::fmt::Debug),*> DebuggableTuple for (#(#type_params,)*)
            {
                fn debug_tuple(&self) -> String {
                    format!(#format_str, #(#debug_refs),*)
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

                impl<Type> HomogeneousColumns<Type> for () {}
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
                impl<#(#type_params: Clone + core::fmt::Debug),*> NonEmptyProjection for (#(#type_params,)*)
                where #first_type: TypedColumn, #(#projection_bounds),*
                {
                    type Table = <#first_type as diesel::Column>::Table;
                }

                impl<#(#type_params: TypedColumn),*> Columns for (#(#type_params,)*)
                {
                    type Types = (#(<#type_params as TypedColumn>::Type,)*);
                    type Tables = (#(<#type_params as diesel::Column>::Table,)*);
                }

                impl<#(#type_params: Clone + core::fmt::Debug),*> Projection<<#first_type as diesel::Column>::Table> for (#(#type_params,)*)
                where #first_type: TypedColumn, #(#projection_bounds),*
                {
                }

                impl<Type, #(#type_params: TypedColumn<Type=Type>),*> HomogeneousColumns<Type> for (#(#type_params,)*)
                {
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

        let maybe_where = if size == 0 {
            None
        } else {
            Some(quote! { where })
        };

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
#[allow(clippy::too_many_lines)]
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
                    <T as crate::set_column::SetColumn<#t>>::set_column(self, &values.#idx);
                }
            })
            .collect();

        let try_set_individual_calls: Vec<_> = type_params
            .iter()
            .zip(&indices)
            .map(|(t, idx)| {
                quote! {
                    <T as crate::set_column::TrySetColumn<#t>>::try_set_column(self, &values.#idx)?;
                }
            })
            .collect();

        let may_set_individual_calls: Vec<_> = type_params
            .iter()
            .zip(&indices)
            .map(|(t, idx)| {
                quote! {
                    <T as crate::set_column::MaySetColumn<#t>>::may_set_column(self, values.#idx);
                }
            })
            .collect();

        let try_may_set_individual_calls: Vec<_> = type_params
            .iter()
            .zip(&indices)
            .map(|(t, idx)| {
                quote! {
                    if let Some(value) = values.#idx {
                        <T as crate::set_column::TrySetColumn<#t>>::try_set_column(self, value)?;
                    }
                }
            })
            .collect();

        let value_replicates = type_params
            .iter()
            .map(|_| quote! { value })
            .collect::<Vec<_>>();

        quote! {
            impl<T, #(#type_params),*> GetColumns<(#(#type_params,)*)> for T
            where T: GetColumn<#first_type>, #(T: GetColumn<#type_params>),*,
                    #first_type: TypedColumn, #(#type_params: TypedColumn),*
            {
                #[inline]
                fn get_columns(&self) -> <<(#(#type_params,)*) as Columns>::Types as crate::RefTuple>::Output<'_> {
                    (#(<T as GetColumn<#type_params>>::get_column(self),)*)
                }
            }

            impl<T, #(#type_params),*> MayGetColumns<(#(#type_params,)*)> for T
            where T: MayGetColumn<#first_type>, #(T: MayGetColumn<#type_params>),*,
                    #first_type: TypedColumn, #(#type_params: TypedColumn),*
            {
                #[inline]
                fn may_get_columns(&self) -> <<<(#(#type_params,)*) as Columns>::Types as crate::RefTuple>::Output<'_> as OptionTuple>::Output {
                    (#(<T as MayGetColumn<#type_params>>::may_get_column(self),)*)
                }
            }

            impl<T, #(#type_params),*> SetColumns<(#(#type_params,)*)> for T
                where
                    #(T: crate::set_column::SetColumn<#type_params>,)*
                    #(#type_params: TypedColumn,)*
            {
                #[inline]
                fn set_columns(&mut self, values: <<(#(#type_params,)*) as Columns>::Types as crate::RefTuple>::Output<'_>) -> &mut Self {
                    #(#set_individual_calls)*
                    self
                }
            }

            impl<T, #(#type_params: TypedColumn),*> MaySetColumns<(#(#type_params,)*)> for T
            where
                #(T: crate::set_column::MaySetColumn<#type_params>,)*
            {
                #[inline]
                fn may_set_columns(&mut self, values: <<<(#(#type_params,)*) as Columns>::Types as crate::RefTuple>::Output<'_> as OptionTuple>::Output) -> &mut Self {
                    #(#may_set_individual_calls)*
                    self
                }
            }

            impl<T: SetColumns<(#(#type_params,)*)>, Type: core::fmt::Debug + Clone, #(#type_params: TypedColumn<Type = Type>),*> SetHomogeneousColumn<Type, (#(#type_params,)*)> for T
            {
                #[inline]
                fn set_homogeneous_columns(&mut self, value: &Type) -> &mut Self {
                    <T as SetColumns<(#(#type_params,)*)>>::set_columns(self, (#(#value_replicates,)*))
                }
            }

            impl<T, #(#type_params: TypedColumn,)*> TrySetColumns<(#(#type_params,)*)> for T
            where
                #(T: crate::set_column::TrySetColumn<#type_params>,)*
            {
                #[inline]
                fn try_set_columns(&mut self, values: <<(#(#type_params,)*) as Columns>::Types as crate::RefTuple>::Output<'_>) -> anyhow::Result<&mut Self> {
                    #(#try_set_individual_calls)*
                    Ok(self)
                }
            }

            impl<T: TrySetColumns<(#(#type_params,)*)>, Type: core::fmt::Debug + Clone, #(#type_params: TypedColumn<Type = Type>),*> TrySetHomogeneousColumn<Type, (#(#type_params,)*)> for T
            {
                #[inline]
                fn try_set_homogeneous_columns(&mut self, value: &Type) -> anyhow::Result<&mut Self> {
                    <T as TrySetColumns<(#(#type_params,)*)>>::try_set_columns(self, (#(#value_replicates,)*))
                }
            }

            impl<T, #(#type_params: TypedColumn),*> TryMaySetColumns<(#(#type_params,)*)> for T
            where
                #(T: crate::set_column::TrySetColumn<#type_params>,)*
            {
                #[inline]
                fn try_may_set_columns(&mut self, values: <<<(#(#type_params,)*) as Columns>::Types as crate::RefTuple>::Output<'_> as OptionTuple>::Output) -> anyhow::Result<&mut Self> {
                    #(#try_may_set_individual_calls)*
                    Ok(self)
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

                #[inline]
                fn nested_insert_tuple(self, conn: &mut Conn) -> anyhow::Result<Self::ModelsTuple> {
                    Ok((#(self.#indices_tokens.insert(conn)?,)*))
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
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();

        let option_model_types: Vec<_> = type_params
            .iter()
            .map(|t| {
                quote! { Option<<<#t as HasTable>::Table as TableAddition>::Model> }
            })
            .collect();
        let indices_tokens: Vec<_> = indices.iter().collect();

        quote! {
            impl<Conn, #(#type_params,)*> NestedInsertOptionTuple<Conn> for (#(Option<#type_params>,)*)
            where
                Conn: LoadConnection,
                #(#type_params: NestedInsert<Conn> + HasTableAddition,)*
            {
                type OptionModelsTuple = (#(#option_model_types,)*);

                fn nested_insert_option_tuple(self, conn: &mut Conn) -> anyhow::Result<Self::OptionModelsTuple> {
                    Ok((#(match self.#indices_tokens {
                        Some(builder) => Some(builder.insert(conn)?),
                        None => None,
                    },)*))
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
            }
        }
    });

    quote! {
        #impls
    }
}

/// Generate BundlableTables trait implementations
pub fn generate_bundlable_tables() -> TokenStream {
    let impls = generate_all_sizes(|size| {
        let type_params = type_params(size);

        quote! {
            impl<#(#type_params: BundlableTable),*> BundlableTables for (#(#type_params,)*)
            {
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
                    self.#idx.get_column()
                }
            })
            .collect();

        // For may_get_primary_keys, build extractors for each optional model
        let pk_extractors: Vec<_> = type_params
            .iter()
            .zip(&indices)
            .map(|(t, idx)| {
                quote! {
                    optional_self.#idx.as_ref().map(|model: &#t| model.get_column())
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
                #[inline]
                fn get_primary_keys(&self) -> <<<Self::Tables as NonCompositePrimaryKeyTables>::PrimaryKeys as Columns>::Types as crate::RefTuple>::Output<'_> {
                    (#(#get_pk_calls,)*)
                }

                #[inline]
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

        quote! {
            impl<#(#type_params: BundlableTable),*> BuilderBundles for (#(TableBuilderBundle<#type_params>,)*)
            {
                type CompletedBundles = (#(CompletedTableBuilderBundle<#type_params>,)*);

                #[inline]
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

        // Generate TableNotEqual bounds: T: TableNotEqual<A1>, T: TableNotEqual<A2>, etc.
        let table_not_equal_bounds: Vec<_> = type_params
            .iter()
            .map(|t| {
                quote! { T: diesel::query_source::TableNotEqual<#t> }
            })
            .collect();

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
    });

    quote! {
        #impls
    }
}

/// Generate HorizontalSameAsKeys trait implementations
pub fn generate_horizontal_same_as_keys() -> TokenStream {
    let impls = generate_all_sizes(|size| {
        let type_params = type_params(size);

        let additional_requirements = if size > 0 {
            Some(quote! {+ HasPrimaryKey })
        } else {
            None
        };

        quote! {
            impl<T, #(#type_params),*> HorizontalSameAsKeys<T> for (#(#type_params,)*)
            where
                T: diesel::Table #additional_requirements,
                #(#type_params: HorizontalSameAsKey<Table = T>,)*
            {
                type ReferencedTables = (#(<#type_params as SingletonForeignKey>::ReferencedTable,)*);
                type FirstForeignColumns = (#(<<#type_params as HorizontalSameAsKey>::ForeignColumns as NthIndex<U0>>::NthType,)*);
            }
        }
    });

    quote! {
        #impls
    }
}

/// Generate CompletedTableBuilder NestedInsert trait implementations for all
/// tuple sizes (2-32).
///
/// The size 1 case is handled separately in the main codebase as a base case.
/// This generates implementations for tuples of size 2 and up, where the
/// pattern is:
/// - Pop the first bundle from the tuple
/// - Insert it to get the parent model
/// - Set the foreign key columns in the remaining bundles
/// - Recursively insert the remaining bundles
pub fn generate_completed_table_builder_nested_insert() -> TokenStream {
    // Start from size 2 since size 1 is the base case handled separately
    let impls = (2..=MAX_TUPLE_SIZE).map(|size| {
        let type_params = type_params(size);
        let first_type = &type_params[0];
        let remaining_types = &type_params[1..];

        // Build the full tuple type
        let full_tuple = quote! { (#(CompletedTableBuilderBundle<#type_params>,)*) };

        quote! {
            impl<Conn, T, #(#type_params),*> NestedInsert<Conn>
                for CompletedTableBuilder<T, #full_tuple>
            where
                Conn: diesel::connection::LoadConnection,
                T: BuildableTable + HasPrimaryKey,
                #(T: DescendantOf<#type_params>,)*
                #(#type_params: AncestralBuildableTable<T>,)*
                CompletedTableBuilderBundle<#first_type>: NestedInsert<Conn, Table = #first_type>,
                #full_tuple: TypedFirst<
                    CompletedTableBuilderBundle<#first_type>,
                    PopOutput = (#(CompletedTableBuilderBundle<#remaining_types>,)*),
                >,
                CompletedTableBuilder<T, (#(CompletedTableBuilderBundle<#remaining_types>,)*)>: NestedInsert<Conn, Table = T>
                    + TrySetHomogeneousColumn<
                        <<#first_type as Table>::PrimaryKey as TypedColumn>::Type,
                        <<#first_type as Table>::PrimaryKey as VerticalSameAsGroup<T>>::VerticalSameAsColumns,
                    >,
            {
                #[inline]
                fn insert(self, conn: &mut Conn) -> anyhow::Result<<T as TableAddition>::Model> {
                    use typed_tuple::prelude::TypedTuple;
                    let (first, bundles) = self.bundles.pop();
                    let model: <#first_type as TableAddition>::Model = first.insert(conn)?;
                    let mut next_builder: CompletedTableBuilder<T, _> =
                        CompletedTableBuilder { bundles, table: PhantomData };
                    next_builder.try_set_homogeneous_columns(model.get_column())?;
                    next_builder.insert(conn)
                }
            }
        }
    }).collect::<proc_macro2::TokenStream>();

    quote! {
        #impls
    }
}

/// Generate `HorizontalSameAsColumns` trait implementations for all tuple sizes
/// (1-32).
pub fn generate_horizontal_same_as_columns() -> TokenStream {
    let impls = (1..=MAX_TUPLE_SIZE).map(|size| {
        let type_params = type_params(size);
        let host_column_params = type_params.iter().enumerate().map(|(i, _)| {
            syn::Ident::new(&format!("HC{i}"), proc_macro2::Span::call_site())
        }).collect::<Vec<_>>();
        // Generate bounds: T0: HorizontalSameAsColumn<Key, HC0>, T1: HorizontalSameAsColumn<Key, HC1>, etc.
        let column_impl_bounds = type_params.iter().zip(host_column_params.iter()).map(|(col, hc)| {
            quote! {
                #col: HorizontalSameAsColumn<Key, #hc, Table = <Key as SingletonForeignKey>::ReferencedTable>
            }
        });
        // Generate bounds that each host column is on the Key's table
        let host_column_table_bounds = host_column_params.iter().zip(type_params.iter()).map(|(hc, tp)| {
            quote! {
                #hc: TypedColumn<Table = <Key as Column>::Table, Type = <#tp as TypedColumn>::Type>
            }
        });
        quote! {
            impl<Key, #(#host_column_params,)* #(#type_params),*> HorizontalSameAsColumns<Key, (#(#host_column_params,)*)> for (#(#type_params,)*)
            where
                Key: HorizontalSameAsKey<HostColumns = (#(#host_column_params,)*), ForeignColumns = (#(#type_params,)*)>,
                #(#host_column_table_bounds,)*
                #(#column_impl_bounds,)*
                (#(#type_params,)*): NonEmptyProjection<Table = <Key as SingletonForeignKey>::ReferencedTable, Types = <(#(#host_column_params,)*) as Columns>::Types>
                    + NthIndex<
                        U0,
                        NthType: TypedColumn<
                            Type = <<<Key as Column>::Table as Table>::PrimaryKey as TypedColumn>::Type,
                            Table = <Key as SingletonForeignKey>::ReferencedTable,
                        >,
                    >,
            {
            }
        }
    }).collect::<TokenStream>();

    quote! {
        #impls
    }
}

/// Generate `TrySetMandatorySameAsColumns` and
/// `TrySetDiscretionarySameAsColumns` trait implementations for all tuple sizes
/// (0-32).
pub fn generate_try_set_same_as_columns() -> TokenStream {
    // Generate empty tuple implementation
    let empty_impl = quote! {
        impl<Type, T: HasTable> TrySetMandatorySameAsColumns<Type, (), ()> for T
        {
            #[inline]
            fn try_set_mandatory_same_as_columns(&mut self, _value: &Type) -> anyhow::Result<&mut Self> {
                Ok(self)
            }
        }

        impl<Type, T: HasTable> TryMaySetDiscretionarySameAsColumns<Type, (), ()> for T
        {
            #[inline]
            fn try_may_set_discretionary_same_as_columns(&mut self, _value: &Type) -> anyhow::Result<&mut Self> {
                Ok(self)
            }
        }
    };

    // Generate implementations for tuples of size 1-32
    let tuple_impls = (1..=MAX_TUPLE_SIZE).map(|size| {
        let keys = type_params(size);
        let column_types = keys.iter().map(|key| {
            quote! {
                <<#key as HorizontalSameAsKey>::ForeignColumns as typed_tuple::prelude::NthIndex<typed_tuple::prelude::U0>>::NthType
            }
        }).collect::<Vec<_>>();

        // Generate the try_set_mandatory_same_as_column calls
        let mandatory_calls = keys.iter().zip(column_types.iter()).map(|(key, column_type)| {
            quote! {
                <
                    Self as TrySetMandatorySameAsColumn<
                        #key,
                        #column_type
                    >
                >::try_set_mandatory_same_as_column(self, value)?;
            }
        });
        // Generate the try_set_discretionary_same_as_column calls
        let discretionary_calls = keys.iter().zip(column_types.iter()).map(|(key, column_type)| {
            quote! {
                <
                    Self as TryMaySetDiscretionarySameAsColumn<
                        #key,
                        #column_type
                    >
                >::try_may_set_discretionary_same_as_column(self, value)?;
            }
        });
        // Generate where clauses for TrySetMandatorySameAsColumn
        let mandatory_trait_bounds = keys.iter().zip(column_types.iter()).map(|(key, column_type)| {
            quote! {
                Self: TrySetMandatorySameAsColumn<#key, #column_type>
            }
        });
        // Generate where clauses for TryMaySetDiscretionarySameAsColumn
        let discretionary_trait_bounds = keys.iter().zip(column_types.iter()).map(|(key, column_type)| {
            quote! {
                Self: TryMaySetDiscretionarySameAsColumn<#key, #column_type>
            }
        });

        quote! {
            impl<
                #(#keys: MandatorySameAsIndex<Table = T>,)*
                T: BundlableTable + HasPrimaryKey
            > TrySetMandatorySameAsColumns<
                <<T as Table>::PrimaryKey as TypedColumn>::Type,
                (#(#keys,)*),
                (#(#column_types,)*)
            > for CompletedTableBuilderBundle<T>
            where
                #(#mandatory_trait_bounds,)*
            {
                #[inline]
                fn try_set_mandatory_same_as_columns(
                    &mut self,
                    value: &<<T as diesel::Table>::PrimaryKey as TypedColumn>::Type
                ) -> anyhow::Result<&mut Self> {
                    #(#mandatory_calls)*
                    Ok(self)
                }
            }

            impl<
                #(#keys: DiscretionarySameAsIndex<Table = T>,)*
                T: BundlableTable + HasPrimaryKey
            > TryMaySetDiscretionarySameAsColumns<
                <<T as Table>::PrimaryKey as TypedColumn>::Type,
                (#(#keys,)*),
                (#(#column_types,)*)
            > for CompletedTableBuilderBundle<T>
            where
                #(#discretionary_trait_bounds,)*
            {
                #[inline]
                fn try_may_set_discretionary_same_as_columns(
                    &mut self,
                    value: &<<Self::Table as diesel::Table>::PrimaryKey as TypedColumn>::Type
                ) -> anyhow::Result<&mut Self> {
                    #(#discretionary_calls)*
                    Ok(self)
                }
            }
        }
    }).collect::<TokenStream>();

    quote! {
        #empty_impl
        #tuple_impls
    }
}

/// Generate `TableIndex` trait marker implementations for all tuple sizes (1-32).
pub fn generate_table_index() -> proc_macro2::TokenStream {
    let impls: TokenStream = (1..=MAX_TUPLE_SIZE)
        .map(|size| {
            let type_params = type_params(size);

            // Generate index constants (U0, U1, U2, ...)
            let indices: Vec<_> = (0..size)
                .map(|i| {
                    let ident = quote::format_ident!("U{}", i);
                    quote! { typed_tuple::prelude::#ident }
                })
                .collect();

            // Generate IndexedColumn bounds for each column
            let indexed_column_bounds =
                type_params.iter().zip(indices.iter()).map(|(param, idx)| {
                    quote! {
                        #param: IndexedColumn<#idx, (#(#type_params,)*)>
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
        .collect();

    quote! {
        #impls
    }
}

/// Generate `ForeignKey` trait marker implementations for all tuple sizes (1-32).
pub fn generate_foreign_key() -> proc_macro2::TokenStream {
    let impls: TokenStream = (1..=MAX_TUPLE_SIZE)
        .map(|size| {
            let host_params = type_params(size);
            let ref_params: Vec<_> = (1..=size)
                .map(|i| {
                    let ident = quote::format_ident!("R{}", i);
                    quote! { #ident }
                })
                .collect();
            // Generate index constants (U0, U1, U2, ...)
            let indices: Vec<_> = (0..size)
                .map(|i| {
                    let ident = quote::format_ident!("U{}", i);
                    quote! { typed_tuple::prelude::#ident }
                })
                .collect();
            // Generate HostColumn bounds for each host parameter with type equality
            let host_column_bounds = host_params.iter().zip(ref_params.iter()).zip(indices.iter()).map(|((host_param, ref_param), idx)| {
                quote! {
                    #host_param: HostColumn<#idx, (#(#host_params,)*), (#(#ref_params,)*)> + TypedColumn<Type = <#ref_param as TypedColumn>::Type>
                }
            });
            // Generate IndexedColumn bounds for each referenced parameter
            let indexed_column_bounds = ref_params.iter().zip(indices.iter()).map(|(ref_param, idx)| {
                quote! {
                    #ref_param: IndexedColumn<#idx, (#(#ref_params,)*)>
                }
            });

            quote! {
                impl<#(#host_params: TypedColumn,)* #(#ref_params: TypedColumn,)*> ForeignKey<(#(#ref_params,)*)> for (#(#host_params,)*)
                where
                    (#(#host_params,)*): NonEmptyProjection,
                    (#(#ref_params,)*): NonEmptyProjection,
                    #(#host_column_bounds,)*
                    #(#indexed_column_bounds,)*
                {
                }
            }
        })
        .collect();

    quote! {
        #impls
    }
}
