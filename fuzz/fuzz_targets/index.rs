//! Fuzzes the shared `index!` / `unique_index!` expansion.
//!
//! Input encoding: the token stream inside an `index!(...)` or
//! `unique_index!(...)` invocation, a comma-terminated list of column paths
//! with an optional trailing comma. The marker trait is chosen from the input
//! length so both `IndexedColumn` and `UniquelyIndexedColumn` are exercised.
//! The oracle re-derives the documented expansion from the parsed columns and
//! requires the native expansion to match it token for token. Each column at
//! position `i` must produce `impl #trait<::diesel_builders::typenum::U{i},
//! (#columns, )> for #column {}`.

#![no_main]

use libfuzzer_sys::fuzz_target;
use quote::ToTokens;
use syn::parse::Parser;

fuzz_target!(|input: &str| {
    let body: proc_macro2::TokenStream = match input.parse() {
        Ok(tokens) => tokens,
        Err(_) => return,
    };
    let trait_path: proc_macro2::TokenStream = if input.len() % 2 == 0 {
        quote::quote! { ::diesel_builders::IndexedColumn }
    } else {
        quote::quote! { ::diesel_builders::UniquelyIndexedColumn }
    };

    // The reference parse snapshots the body so both branches can still
    // consume it in the native expansion. A column is a path, not any type:
    // a trait-bound type such as `A + B` cannot legally appear as an impl
    // target, so expand_index only accepts a terminated list of paths. A
    // single reserved-keyword segment such as `try` still parses as a
    // syn::Path here, but expand_index's own self-check rejects it too,
    // since it can never legally appear as an impl target either; that is
    // a legitimate rejection this reference parse alone cannot predict.
    let parsed_columns: syn::punctuated::Punctuated<syn::Path, syn::Token![,]> =
        match syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated.parse2(body.clone()) {
            Ok(columns) => columns,
            Err(_) => {
                diesel_builders_derive_fuzz::expand_index(body, &trait_path)
                    .expect_err("a body that is not a terminated path list must be rejected");
                return;
            }
        };
    let columns: Vec<syn::Path> = parsed_columns.into_iter().collect();

    let tokens = match diesel_builders_derive_fuzz::expand_index(body, &trait_path) {
        Ok(tokens) => tokens,
        Err(_) => return,
    };
    let parsed: syn::File = syn::parse2(tokens).expect("expansion must be valid Rust");

    let column_tuple: proc_macro2::TokenStream = quote::quote! { ( #(#columns,)* ) };
    assert_eq!(parsed.items.len(), columns.len(), "one marker impl per column");
    for (item, (position, column)) in parsed.items.iter().zip(columns.iter().enumerate()) {
        let syn::Item::Impl(impl_item) = item else {
            panic!("every expansion item must be a marker impl");
        };
        let position: syn::Type =
            syn::parse_str(&format!("::diesel_builders::typenum::U{position}"))
                .expect("typenum marker position parses");
        let expected = quote::quote! {
            impl #trait_path<#position, #column_tuple> for #column {}
        };
        assert_eq!(
            impl_item.to_token_stream().to_string(),
            expected.to_string(),
            "marker impl must carry the column, its position, and the full column tuple"
        );
    }
});
