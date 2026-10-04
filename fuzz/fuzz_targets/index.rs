//! Fuzzes the shared `index!` / `unique_index!` expansion.
//!
//! Input encoding: the token stream inside an `index!(...)` or
//! `unique_index!(...)` invocation, a comma-terminated list of column types
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
    // consume it in the native expansion.
    let parsed_columns: syn::punctuated::Punctuated<syn::Type, syn::Token![,]> =
        match syn::punctuated::Punctuated::<syn::Type, syn::Token![,]>::parse_terminated.parse2(body.clone()) {
            Ok(columns) => columns,
            Err(_) => {
                diesel_builders_derive_fuzz::expand_index(body, &trait_path)
                    .expect_err("a body that is not a terminated type list must be rejected");
                return;
            }
        };
    let columns: Vec<syn::Type> = parsed_columns.into_iter().collect();

    let tokens = diesel_builders_derive_fuzz::expand_index(body, &trait_path)
        .expect("a terminated type list must expand");
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
