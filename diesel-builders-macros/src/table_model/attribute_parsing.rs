//! Attribute parsing utilities for `TableModel` derive.

use syn::{DeriveInput, Ident};

/// Extract the table name from the `#[diesel(table_name = ...)]` attribute.
pub fn extract_table_name(input: &DeriveInput) -> syn::Result<Ident> {
    let table_name = input.attrs.iter().find_map(|attr| {
        if !attr.path().is_ident("diesel") {
            return None;
        }

        let mut table_name = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("table_name") {
                let value = meta.value()?;
                let lit: syn::Ident = value.parse()?;
                table_name = Some(lit);
                Ok(())
            } else {
                Ok(())
            }
        });
        table_name
    });

    table_name.ok_or_else(|| {
        syn::Error::new_spanned(
            input,
            "TableModel derive requires a #[diesel(table_name = ...)] attribute",
        )
    })
}

/// Extract primary key columns from `#[diesel(primary_key = ...)]` attribute.
/// Defaults to "id" if not specified.
pub fn extract_primary_key_columns(input: &DeriveInput) -> Vec<Ident> {
    input
        .attrs
        .iter()
        .find_map(|attr| {
            if !attr.path().is_ident("diesel") {
                return None;
            }

            let mut pk_columns = Vec::new();
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("primary_key") {
                    // Parse primary_key(col1, col2, ...)
                    let content;
                    syn::parenthesized!(content in meta.input);
                    let punct: syn::punctuated::Punctuated<syn::Ident, syn::Token![,]> =
                        syn::punctuated::Punctuated::parse_terminated(&content)?;
                    pk_columns.extend(punct);
                    Ok(())
                } else {
                    Ok(())
                }
            });

            if pk_columns.is_empty() {
                None
            } else {
                Some(pk_columns)
            }
        })
        .unwrap_or_else(|| {
            // Default: if no primary_key attribute, assume "id" is the primary key
            vec![syn::Ident::new("id", proc_macro2::Span::call_site())]
        })
}

/// Extract the insertable model name from `#[table_model(insertable = ...)]` attribute.
/// Defaults to "New{ModelName}" if not specified.
pub fn extract_insertable_name(input: &DeriveInput, struct_ident: &Ident) -> Ident {
    input
        .attrs
        .iter()
        .find_map(|attr| {
            if !attr.path().is_ident("table_model") {
                return None;
            }

            let mut found: Option<syn::Ident> = None;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("insertable") {
                    let value = meta.value()?;
                    if let Ok(id) = value.parse::<syn::Ident>() {
                        found = Some(id);
                        return Ok(());
                    }
                    if let Ok(lit) = value.parse::<syn::LitStr>() {
                        found = Some(syn::Ident::new(&lit.value(), lit.span()));
                        return Ok(());
                    }
                }
                Ok(())
            });

            found
        })
        .unwrap_or_else(|| {
            syn::Ident::new(
                &format!("New{struct_ident}"),
                proc_macro2::Span::call_site(),
            )
        })
}
