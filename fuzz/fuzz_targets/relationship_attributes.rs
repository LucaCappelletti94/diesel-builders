//! Fuzzes the owned relationship-attribute parsing through the derive
//! expansion.
//!
//! Input encoding: one `#[...]` attribute per line; every line must parse as
//! a `syn::Attribute`. Each attribute is applied to the container and to
//! every field of a fixed two-field template struct. The target runs the
//! owned attribute parsers, the field validation, and the full expansion,
//! and requires that invalid or conflicting attributes never expand and that
//! accepted inputs expand to a non-empty, syntactically valid `syn::File`.

#![no_main]

use libfuzzer_sys::fuzz_target;
use syn::parse::Parser;

const TEMPLATE: &str = "struct Model { id: i32, other: i32 }";

fuzz_target!(|input: &str| {
    let template: syn::DeriveInput = match syn::parse_str(TEMPLATE) {
        Ok(parsed) => parsed,
        Err(_) => return,
    };

    let mut attributes = Vec::new();
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match syn::Attribute::parse_outer.parse_str(line) {
            Ok(mut parsed) if parsed.len() == 1 => attributes.append(&mut parsed),
            Ok(_) | Err(_) => return,
        }
    }
    if attributes.is_empty() {
        return;
    }

    let mut model = template;
    let syn::Data::Struct(data) = &mut model.data else {
        return;
    };
    let syn::Fields::Named(fields) = &mut data.fields else {
        return;
    };
    for attribute in &attributes {
        model.attrs.push(attribute.clone());
    }
    for field in fields.named.iter_mut() {
        for attribute in &attributes {
            field.attrs.push(attribute.clone());
        }
    }

    let invalid_fields = fields
        .named.iter()
        .any(|field| diesel_builders_derive_fuzz::validate_field_attributes(field).is_err());
    let has_mandatory_discretionary_conflict = fields.named.iter().any(|field| {
        diesel_builders_derive_fuzz::extract_mandatory_table(field)
            .is_ok_and(|table| table.is_some())
            && diesel_builders_derive_fuzz::extract_discretionary_table(field)
                .is_ok_and(|table| table.is_some())
    });

    match diesel_builders_derive_fuzz::derive_table_model(&model) {
        Ok(tokens) => {
            assert!(
                !invalid_fields && !has_mandatory_discretionary_conflict,
                "invalid or conflicting attributes must not expand"
            );
            let parsed: syn::File =
                syn::parse2(tokens).expect("accepted input must expand to valid Rust");
            assert!(!parsed.items.is_empty(), "expansion must emit items");
        }
        Err(_) => {}
    }
});
