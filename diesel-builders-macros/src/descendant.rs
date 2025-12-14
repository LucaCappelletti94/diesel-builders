//! Generates auxiliary implementations for descendant tables in Diesel Builders.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

/// Generates the auxiliary implementations required for a `Descendant` table.
///
/// This includes:
/// - `DescendantOf` implementations for each ancestor.
/// - `GetColumn` implementations for each ancestor's primary key.
/// - `DescendantOf` implementation for the root (if not in ancestors).
/// - `AncestorOfIndex` implementations for each ancestor and self.
pub fn generate_auxiliary_descendant_impls(
    table_type: &Type,
    ancestors: &[Type],
    root_type: &Type,
) -> TokenStream {
    let num_ancestors = ancestors.len();

    // Generate TupleIndex for self (last position in ancestors + self)
    let self_idx = syn::Ident::new(&format!("U{num_ancestors}"), proc_macro2::Span::call_site());

    // Generate DescendantOf implementations for each direct ancestor
    let descendant_of_impls: Vec<_> = ancestors
        .iter()
        .map(|ancestor| {
            quote! {
                impl diesel_builders::DescendantOf<#ancestor> for #table_type {}
            }
        })
        .collect();

    // Generate `GetColumn` implementation for each ancestor's primary key
    // for the descendant table model
    let descendant_table_model = quote! {
        <#table_type as diesel_builders::TableExt>::Model
    };
    let get_column_impls: Vec<_> = ancestors
        .iter()
        .map(|ancestor| {
            quote! {
                impl diesel_builders::GetColumn<<#ancestor as diesel::Table>::PrimaryKey>
                    for #descendant_table_model
                {
                    fn get_column_ref(
                        &self,
                    ) -> &<<#ancestor as diesel::Table>::PrimaryKey as diesel_builders::Typed>::ColumnType {
                        use diesel::Identifiable;
                        self.id()
                    }
                }
            }
        })
        .collect();

    // Generate DescendantOf implementation for the root (if it's not already in
    // ancestors) We need to check if root_type is different from all ancestors
    let root_descendant_of_impl = if ancestors.is_empty() {
        quote! {}
    } else {
        // Check if the root is already in the ancestors list by comparing token streams
        let root_tokens = quote! { #root_type }.to_string();
        let is_root_in_ancestors = ancestors.iter().any(|ancestor| {
            let ancestor_tokens = quote! { #ancestor }.to_string();
            ancestor_tokens == root_tokens
        });

        if is_root_in_ancestors {
            quote! {}
        } else {
            quote! {
                impl diesel_builders::DescendantOf<#root_type> for #table_type {}
            }
        }
    };

    // Generate AncestorOfIndex implementations for each ancestor
    let ancestor_of_index_impls: Vec<_> = ancestors
        .iter()
        .enumerate()
        .map(|(i, ancestor)| {
            let idx = syn::Ident::new(&format!("U{i}"), proc_macro2::Span::call_site());
            quote! {
                impl diesel_builders::AncestorOfIndex<#table_type> for #ancestor {
                    type Idx = diesel_builders::typenum::#idx;
                }
            }
        })
        .collect();

    // Generate AncestorOfIndex for self
    let self_ancestor_of_index = quote! {
        impl diesel_builders::AncestorOfIndex<#table_type> for #table_type {
            type Idx = diesel_builders::typenum::#self_idx;
        }
    };

    quote! {
        #(#descendant_of_impls)*

        #root_descendant_of_impl

        #self_ancestor_of_index

        #(#get_column_impls)*

        #(#ancestor_of_index_impls)*
    }
}
