//! Generates auxiliary implementations for descendant tables in Diesel
//! Builders.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

/// Generates the auxiliary implementations required for a `Descendant` table.
pub fn generate_auxiliary_descendant_impls(table_type: &Type, ancestors: &[Type]) -> TokenStream {
    assert!(!ancestors.contains(table_type), "Table cannot be its own ancestor");

    let num_ancestors = ancestors.len();

    // Generate TupleIndex for self (last position in ancestors + self)
    let self_idx = crate::utils::typenum_ident(num_ancestors);

    // Generate DescendantOf implementations for each direct ancestor
    let descendant_of_impls: Vec<_> = ancestors
        .iter()
        .map(|ancestor| {
            quote! {
                impl diesel_builders::DescendantOf<#ancestor> for #table_type {}
            }
        })
        .collect();

    // Generate AncestorOfIndex implementations for each ancestor
    let ancestor_of_index_impls: Vec<_> = ancestors
        .iter()
        .enumerate()
        .map(|(i, ancestor)| {
            let idx = crate::utils::typenum_ident(i);
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

    // Generate the `NestedInnerJoin` implementation for this table, joining the
    // table with each of its ancestors. The join query type is a foreign
    // tuple-covered type and its construction relies on diesel's private
    // `Inner` join kind marker, so it cannot be produced by a generic library
    // impl (that would violate the orphan rule and name a private item).
    // Emitting it here, on the concrete local table type, sidesteps both: the
    // self type is local, and building the join through `QueryDsl::inner_join`
    // and the public `InnerJoin` alias keeps the marker unnamed while the
    // compiler discharges the join bound itself.
    let join_query_ty = ancestors.iter().rev().fold(quote! { #table_type }, |acc, ancestor| {
        quote! { ::diesel::helper_types::InnerJoin<#acc, #ancestor> }
    });
    let join_query_expr = ancestors.iter().rev().fold(
        quote! { <#table_type as ::core::default::Default>::default() },
        |acc, ancestor| {
            quote! {
                ::diesel::QueryDsl::inner_join(
                    #acc,
                    <#ancestor as ::core::default::Default>::default(),
                )
            }
        },
    );

    quote! {
        #(#descendant_of_impls)*

        #self_ancestor_of_index

        #(#ancestor_of_index_impls)*

        impl ::diesel_builders::load_nested_query_builder::NestedInnerJoin for #table_type {
            type JoinQuery = #join_query_ty;

            fn nested_inner_join() -> Self::JoinQuery {
                #join_query_expr
            }
        }
    }
}
