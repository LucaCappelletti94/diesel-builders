//! Fuzzes the full `TableModel` derive expansion from raw derive-input
//! source.
//!
//! Input encoding: Rust source of a struct definition carrying any
//! `#[diesel]` or `#[table_model]` container attributes. The target parses
//! the input with `syn::DeriveInput`, runs the owned expansion, and requires
//! accepted inputs to expand to a syntactically valid `syn::File` that
//! declares the `diesel::table!` definition and binds the input model through
//! the `TableExt` implementation, the same contract the native tests check.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: &str| {
    let derive_input: syn::DeriveInput = match syn::parse_str(input) {
        Ok(parsed) => parsed,
        Err(_) => return,
    };

    match diesel_builders_derive_fuzz::derive_table_model(&derive_input) {
        Ok(tokens) => {
            let parsed: syn::File =
                syn::parse2(tokens).expect("accepted input must expand to valid Rust");
            let model = derive_input.ident.to_string();
            assert!(declares_input_table(&parsed), "expansion must declare the input table");
            assert!(
                implements_table_ext_for_model(&parsed, &model),
                "expansion must bind the input model through `TableExt`"
            );
        }
        Err(_) => {}
    }
});

/// Checks that the expansion contains the `diesel::table!` definition.
fn declares_input_table(file: &syn::File) -> bool {
    file.items.iter().any(|item| {
        let syn::Item::Macro(macro_item) = item else {
            return false;
        };
        let segments = &macro_item.mac.path.segments;
        segments.len() == 2
            && segments[0].ident == "diesel"
            && segments[1].ident == "table"
    })
}

/// Checks that the expansion implements `TableExt` binding the input model.
fn implements_table_ext_for_model(file: &syn::File, model: &str) -> bool {
    file.items.iter().any(|item| {
        let syn::Item::Impl(impl_item) = item else {
            return false;
        };
        let Some((_, trait_path, _)) = impl_item.trait_.as_ref() else {
            return false;
        };
        if trait_path
            .segments
            .last()
            .is_none_or(|segment| segment.ident != "TableExt")
        {
            return false;
        }
        impl_item.items.iter().any(|impl_item| {
            let syn::ImplItem::Type(type_item) = impl_item else {
                return false;
            };
            if type_item.ident != "Model" {
                return false;
            }
            if let syn::Type::Path(type_path) = &type_item.ty {
                type_path.path.get_ident().is_some_and(|ident| ident == model)
            } else {
                false
            }
        })
    })
}
