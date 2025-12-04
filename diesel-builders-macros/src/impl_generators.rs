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
    generate_all_sizes(|size| {
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
    })
}

/// Generate OptionTuple and TransposeOptionTuple trait implementations
pub fn generate_option_tuple() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();

        quote! {
            impl<#(#type_params: Clone + core::fmt::Debug),*> OptionTuple for (#(#type_params,)*) {
                type Output = (#(Option<#type_params>,)*);

                #[inline]
                fn into_option(self) -> Self::Output {
                    (#(Some(self.#indices),)*)
                }
            }

            impl<#(#type_params: Clone + core::fmt::Debug),*> TransposeOptionTuple for (#(Option<#type_params>,)*) {
                type Transposed = (#(#type_params,)*);

                #[inline]
                fn transpose_option(self) -> Option<Self::Transposed> {
                    Some((#(self.#indices?,)*))
                }
            }
        }
    })
}

/// Generate RefTuple trait implementations
pub fn generate_ref_tuple() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        quote! {
            impl<#(#type_params: Clone + core::fmt::Debug,)*> RefTuple for (#(#type_params,)*) {
                type Output<'a> = (#(&'a #type_params,)*) where Self: 'a;
            }
        }
    })
}

/// Generate ClonableTuple trait implementations for all tuple sizes
pub fn generate_clonable_tuple() -> TokenStream {
    generate_all_sizes(|size| {
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
    })
}

/// Generate CopiableTuple trait implementations for all tuple sizes
pub fn generate_copiable_tuple() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();
        let copies = indices.iter().map(|idx| quote! { self.#idx });

        quote! {
            impl<#(#type_params: Copy),*> CopiableTuple for (#(#type_params,)*)
            {
                #[inline]
                fn copy_tuple(&self) -> Self {
                    (#(#copies,)*)
                }
            }
        }
    })
}

/// Generate PartialEqTuple trait implementations for all tuple sizes
pub fn generate_partial_e_tuple() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();
        let eq_checks = indices.iter().map(|idx| quote! { self.#idx == other.#idx });

        quote! {
            impl<#(#type_params: PartialEq),*> PartialEqTuple for (#(#type_params,)*)
            {
                #[inline]
                fn partial_eq_tuple(&self, other: &Self) -> bool {
                    #(#eq_checks && )* true
                }
            }
        }
    })
}

/// Generate EqTuple trait implementations for all tuple sizes
pub fn generate_eq_tuple() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();
        let eq_checks = indices.iter().map(|idx| quote! { self.#idx == other.#idx });

        quote! {
            impl<#(#type_params: Eq),*> EqTuple for (#(#type_params,)*)
            {
                #[inline]
                fn eq_tuple(&self, other: &Self) -> bool {
                    #(#eq_checks && )* true
                }
            }
        }
    })
}

/// Generate HashTuple trait implementations for all tuple sizes
pub fn generate_hash_tuple() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();
        let hash_calls = indices.iter().map(|idx| quote! { self.#idx.hash(state); });

        quote! {
            impl<#(#type_params: std::hash::Hash),*> HashTuple for (#(#type_params,)*)
            {
                #[inline]
                fn hash_tuple<H: std::hash::Hasher>(&self, state: &mut H) {
                    #(#hash_calls)*
                }
            }
        }
    })
}

/// Generate PartialOrdTuple trait implementations for all tuple sizes
pub fn generate_partial_ord_tuple() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices = (0..size).map(syn::Index::from);
        quote! {
            impl<#(#type_params: PartialOrd),*> PartialOrdTuple for (#(#type_params,)*)
            {
                #[inline]
                fn partial_cmp_tuple(&self, other: &Self) -> Option<std::cmp::Ordering> {
                    #(
                        match self.#indices.partial_cmp(&other.#indices) {
                            Some(std::cmp::Ordering::Equal) => {},
                            non_eq => return non_eq,
                        }
                    )*
                    Some(std::cmp::Ordering::Equal)
                }
            }
        }
    })
}

/// Generate OrdTuple trait implementations for all tuple sizes
pub fn generate_ord_tuple() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices = (0..size).map(syn::Index::from);
        quote! {
            impl<#(#type_params: Ord),*> OrdTuple for (#(#type_params,)*)
            {
                #[inline]
                fn cmp_tuple(&self, other: &Self) -> std::cmp::Ordering {
                    #(
                        match self.#indices.cmp(&other.#indices) {
                            std::cmp::Ordering::Equal => {},
                            non_eq => return non_eq,
                        }
                    )*
                    std::cmp::Ordering::Equal
                }
            }
        }
    })
}

/// Generate DebuggableTuple trait implementations for all tuple sizes
pub fn generate_debuggable_tuple() -> TokenStream {
    generate_all_sizes(|size| {
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
    })
}

/// Generate Columns trait implementations
pub fn generate_columns() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);

        quote! {
            impl<#(#type_params: TypedColumn),*> Columns for (#(#type_params,)*)
            {
                type Types = (#(<#type_params as TypedColumn>::Type,)*);
                type Tables = (#(<#type_params as diesel::Column>::Table,)*);
            }

            impl<T: diesel::Table, #(#type_params: TypedColumn<Table=T>),*> Projection<T> for (#(#type_params,)*)
            {
            }

            impl<Type, #(#type_params: TypedColumn<Type=Type>),*> HomogeneousColumns<Type> for (#(#type_params,)*)
            {
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
            }
            impl<#(#type_params: crate::table_addition::HasPrimaryKey),*> NonCompositePrimaryKeyTables for (#(#type_params,)*)
            {
                type PrimaryKeys = (#(<#type_params as diesel::Table>::PrimaryKey,)*);
            }
            impl<#(#type_params: TableModel),*> TableModels for (#(#type_params,)*)
            {
                type Tables = (#(<#type_params as HasTable>::Table,)*);
            }
            impl<#(#type_params: InsertableTableModel),*> InsertableTableModels for (#(#type_params,)*)
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
                fn get_columns(&self) -> <<(#(#type_params,)*) as Columns>::Types as crate::RefTuple>::Output<'_> {
                    (#(<T as GetColumn<#type_params>>::get_column(self),)*)
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
                fn may_get_columns(&self) -> <<<(#(#type_params,)*) as Columns>::Types as crate::RefTuple>::Output<'_> as OptionTuple>::Output {
                    (#(<T as MayGetColumn<#type_params>>::may_get_column(self),)*)
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
                fn set_columns(&mut self, values: <<(#(#type_params,)*) as Columns>::Types as crate::RefTuple>::Output<'_>) -> &mut Self {
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
                fn may_set_columns(&mut self, values: <<<(#(#type_params,)*) as Columns>::Types as crate::RefTuple>::Output<'_> as OptionTuple>::Output) -> &mut Self {
                    #(#may_set_individual_calls)*
                    self
                }
            }
        }
    })
}

/// Generate TrySetColumns trait implementations
pub fn generate_try_set_columns_trait() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();
        let try_set_individual_calls: Vec<_> = type_params
            .iter()
            .zip(&indices)
            .map(|(t, idx)| {
                quote! {
                    <T as TrySetColumn<#t>>::try_set_column(self, values.#idx.clone())?;
                }
            })
            .collect();
        let where_statement = (size > 0).then(|| {
            quote! {
                where
                    #(T: TrySetColumn<#type_params>,)*
                    #(Error: From<<T as TrySetColumn<#type_params>>::Error>,)*
            }
        });

        quote! {
            impl<Error, T: HasTableAddition, #(#type_params: TypedColumn,)*> TrySetColumns<Error, (#(#type_params,)*)> for T
            #where_statement
            {
                #[inline]
                fn try_set_columns(&mut self, values: <<(#(#type_params,)*) as Columns>::Types as crate::RefTuple>::Output<'_>) -> Result<&mut Self, Error> {
                    #(#try_set_individual_calls)*
                    Ok(self)
                }
            }
        }
    })
}

/// Generate TrySetHomogeneousColumn trait implementations
pub fn generate_try_set_homogeneous_column_trait() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let value_replicates = type_params
            .iter()
            .map(|_| quote! { value })
            .collect::<Vec<_>>();
        let type_column_bounds =
            (size > 0).then(|| quote! { #(#type_params: TypedColumn<Type = Type>),* });
        let where_statement = (size > 0).then(|| {
            quote! {
                where
                    T: TrySetColumns<Error, (#(#type_params,)*)>
            }
        });

        quote! {
            impl<Error, T: HasTableAddition, Type: core::fmt::Debug + Clone, #type_column_bounds> TrySetHomogeneousColumn<Error, Type, (#(#type_params,)*)> for T
            #where_statement
            {
                #[inline]
                fn try_set_homogeneous_columns(&mut self, value: &Type) -> Result<&mut Self, Error> {
                    <T as TrySetColumns<Error, (#(#type_params,)*)>>::try_set_columns(self, (#(#value_replicates,)*))
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
                fn try_may_set_columns(&mut self, values: <<<(#(#type_params,)*) as Columns>::Types as crate::RefTuple>::Output<'_> as OptionTuple>::Output) -> Result<&mut Self, Error> {
                    #(#try_may_set_individual_calls)*
                    Ok(self)
                }
            }
        }
    })
}

/// Generate NestedInsertTuple trait implementations
pub fn generate_nested_insert_tuple() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();

        let model_types = type_params
            .iter()
            .map(|t| quote! { <<#t as HasTable>::Table as TableAddition>::Model });

        quote! {
            impl<Error, Conn, #(#type_params),*> NestedInsertTuple<Error, Conn> for (#(#type_params,)*)
            where
                Conn: LoadConnection,
                #(#type_params: crate::RecursiveInsert<Error, Conn> + HasTableAddition,)*
            {
                type ModelsTuple = (#(#model_types,)*);

                #[inline]
                fn nested_insert_tuple(self, conn: &mut Conn) -> crate::BuilderResult<Self::ModelsTuple, Error> {
                    Ok((#(self.#indices.recursive_insert(conn)?,)*))
                }
            }
        }
    })
}

/// Generate NestedInsertOptionTuple trait implementations
pub fn generate_nested_insert_option_tuple() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();

        let option_model_types = type_params
            .iter()
            .map(|t| quote! { Option<<<#t as HasTable>::Table as TableAddition>::Model> });

        quote! {
            impl<Error, Conn, #(#type_params,)*> NestedInsertOptionTuple<Error, Conn> for (#(Option<#type_params>,)*)
            where
                Conn: LoadConnection,
                #(#type_params: crate::RecursiveInsert<Error, Conn> + HasTableAddition,)*
            {
                type OptionModelsTuple = (#(#option_model_types,)*);

                fn nested_insert_option_tuple(self, conn: &mut Conn) -> crate::BuilderResult<Self::OptionModelsTuple, Error> {
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
            (size > 0).then(|| quote! { where #(#type_params: BuildableTable),* });

        quote! {
            impl<#(#type_params),*> crate::BuildableTables for (#(#type_params,)*)
            #where_statement
            {
                type Builders = (#(crate::TableBuilder<#type_params>,)*);
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

/// Generate BuildableColumns trait implementations
pub fn generate_buildable_columns() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);

        quote! {
            impl<#(#type_params),*> BuildableColumns for (#(#type_params,)*)
            where
                Self::Tables: BuildableTables,
                #(#type_params: BuildableColumn),*
            {}
        }
    })
}

/// Generate NonCompositePrimaryKeyTableModels and MayGetPrimaryKeys trait
/// implementations
pub fn generate_table_model() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();

        let get_pk_calls = indices.iter().map(|idx| quote! { self.#idx.get_column() });

        // For may_get_primary_keys, build extractors for each optional model
        let pk_extractors = type_params.iter().zip(&indices).map(|(t, idx)| {
            quote! {
                optional_self.#idx.as_ref().map(|model: &#t| model.get_column())
            }
        });

        // Build the PrimaryKeys type for the where clause
        let primary_key_types = type_params
            .iter()
            .map(|t| quote! { <<#t as HasTable>::Table as diesel::Table>::PrimaryKey });

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
    })
}

/// Generate BuilderBundles trait implementations
pub fn generate_builder_bundles() -> TokenStream {
    generate_all_sizes(|size| {
        let type_params = type_params(size);
        let indices: Vec<_> = (0..size).map(syn::Index::from).collect();

        // Generate try_from calls for each element
        let try_from_calls = indices.iter().map(|idx| {
            quote! {
                CompletedTableBuilderBundle::try_from(self.#idx)?
            }
        });

        quote! {
            impl<#(#type_params: BundlableTable),*> BuilderBundles for (#(TableBuilderBundle<#type_params>,)*)
            {
                type CompletedBundles = (#(CompletedTableBuilderBundle<#type_params>,)*);

                #[inline]
                fn try_complete(self) -> Result<Self::CompletedBundles, crate::IncompleteBuilderError> {
                    Ok((#(#try_from_calls,)*))
                }
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
        let additional_requirements = (size > 0).then(|| quote! {+ HasPrimaryKey });

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
    })
}

/// Generate CompletedTableBuilder NestedInsert trait implementations for all
/// tuple sizes.
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
    (2..=MAX_TUPLE_SIZE).map(|size| {
        let type_params = type_params(size);
        let first_type = &type_params[0];
        let remaining_types = &type_params[1..];
        let last_type = &type_params[size-1];
        // Ancestors are all types except the last one (which is T itself)
        let ancestor_types = &type_params[..size-1];

        // Build the full tuple type
        let full_tuple = quote! { (#(CompletedTableBuilderBundle<#type_params>,)*) };

        quote! {
            impl<Error, Conn, T, #(#type_params),*> RecursiveInsert<Error, Conn>
                for CompletedTableBuilder<T, #full_tuple>
            where
                Conn: diesel::connection::LoadConnection,
                T: BuildableTable + HasPrimaryKey,
                // Only require DescendantOf for ancestor tables, not for T itself
                #(T: DescendantOf<#ancestor_types>,)*
                #(#ancestor_types: AncestralBuildableTable<T>,)*
                #last_type: BundlableTable,
                CompletedTableBuilderBundle<#first_type>: RecursiveInsert<Error, Conn, Table = #first_type>,
                #full_tuple: TypedFirst<
                    CompletedTableBuilderBundle<#first_type>,
                    PopOutput = (#(CompletedTableBuilderBundle<#remaining_types>,)*),
                >,
                CompletedTableBuilder<T, (#(CompletedTableBuilderBundle<#remaining_types>,)*)>: RecursiveInsert<Error, Conn, Table = T>
                    + TrySetHomogeneousColumn<
                        Error,
                        <<#first_type as Table>::PrimaryKey as TypedColumn>::Type,
                        <<#first_type as Table>::PrimaryKey as VerticalSameAsGroup<T>>::VerticalSameAsColumns,
                    >
            {
                #[inline]
                fn recursive_insert(
                    self,
                    conn: &mut Conn
                ) -> BuilderResult<
                    <<Self as HasTable>::Table as TableAddition>::Model,
                    Error
                > {
                    use typed_tuple::prelude::TypedTuple;
                    let (first, bundles) = self.bundles.pop();
                    let model: <#first_type as TableAddition>::Model = first.recursive_insert(conn)?;
                    let mut next_builder: CompletedTableBuilder<T, _> =
                        CompletedTableBuilder { bundles, table: PhantomData };
                    next_builder.try_set_homogeneous_columns(model.get_column()).map_err(BuilderError::Validation)?;
                    next_builder.recursive_insert(conn)
                }
            }
        }
    }).collect::<proc_macro2::TokenStream>()
}

/// Generate `HorizontalSameAsColumns` trait implementations for all tuple sizes.
pub fn generate_horizontal_same_as_columns() -> TokenStream {
    (1..=MAX_TUPLE_SIZE).map(|size| {
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
    }).collect::<TokenStream>()
}

/// Generate `TrySetMandatorySameAsColumns` and
/// `TrySetDiscretionarySameAsColumns` trait implementations for all tuple sizes.
pub fn generate_try_set_same_as_columns() -> TokenStream {
    // Generate implementations for tuples of size 1-32
    let tuple_impls = (1..=MAX_TUPLE_SIZE).map(|size| {
        let keys = type_params(size);
        let column_types = keys.iter().map(|key| {
            quote! {
                <<#key as HorizontalSameAsKey>::ForeignColumns as typed_tuple::prelude::NthIndex<typed_tuple::prelude::U0>>::NthType
            }
        }).collect::<Vec<_>>();

        let first_key = &keys[0];
        let first_column_type = &column_types[0];

        // Generate the try_set_mandatory_same_as_column calls
        let mandatory_calls = keys.iter().zip(column_types.iter()).map(|(key, column_type)| {
            quote! {
                <Self as TrySetMandatorySameAsColumn<#key, #column_type>>::try_set_mandatory_same_as_column(self, value.clone())?;
            }
        });
        // Generate the try_set_discretionary_same_as_column calls
        let discretionary_calls = keys.iter().zip(column_types.iter()).map(|(key, column_type)| {
            quote! {
                <Self as TryMaySetDiscretionarySameAsColumn<#key, #column_type>>::try_may_set_discretionary_same_as_column(self, value.clone())?;
            }
        });

        let mandatory_error_type = quote! {
            <Self as TrySetMandatorySameAsColumn<#first_key, #first_column_type>>::Error
        };

        // Generate where clauses for TrySetMandatorySameAsColumn
        let mandatory_trait_bounds = keys.iter().zip(column_types.iter()).map(|(key, column_type)| {
            if key == first_key {
                quote! {
                    Self: TrySetMandatorySameAsColumn<#key, #column_type>
                }
            } else {
                quote! {
                    Self: TrySetMandatorySameAsColumn<#key, #column_type, Error = #mandatory_error_type>
                }
            }
        });

        let discretionary_error_type = quote! {
            <Self as TryMaySetDiscretionarySameAsColumn<#first_key, #first_column_type>>::Error
        };

        // Generate where clauses for TryMaySetDiscretionarySameAsColumn
        let discretionary_trait_bounds = keys.iter().zip(column_types.iter()).map(|(key, column_type)| {
            if key == first_key {
                quote! {
                    Self: TryMaySetDiscretionarySameAsColumn<#key, #column_type>
                }
            } else {
                quote! {
                    Self: TryMaySetDiscretionarySameAsColumn<#key, #column_type, Error = #discretionary_error_type>
                }
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
                type Error = #mandatory_error_type;

                #[inline]
                fn try_set_mandatory_same_as_columns(
                    &mut self,
                    value: &<<T as diesel::Table>::PrimaryKey as TypedColumn>::Type
                ) -> Result<&mut Self, Self::Error> {
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
                type Error = #discretionary_error_type;

                #[inline]
                fn try_may_set_discretionary_same_as_columns(
                    &mut self,
                    value: &<<Self::Table as diesel::Table>::PrimaryKey as TypedColumn>::Type
                ) -> Result<&mut Self, Self::Error> {
                    #(#discretionary_calls)*
                    Ok(self)
                }
            }
        }
    }).collect::<TokenStream>();

    quote! {
        #tuple_impls
    }
}

/// Generate `TableIndex` trait marker implementations for all tuple sizes.
pub fn generate_table_index() -> proc_macro2::TokenStream {
    (1..=MAX_TUPLE_SIZE)
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
        .collect()
}

/// Generate `ForeignKey` trait marker implementations for all tuple sizes.
pub fn generate_foreign_key() -> proc_macro2::TokenStream {
    (1..=MAX_TUPLE_SIZE)
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
        .collect()
}
