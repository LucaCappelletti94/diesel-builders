//! Submodule with utilities for the diesel-builders macros.

use quote::ToTokens;

/// Convert a `snake_case` string to `CamelCase`.
///
/// This helper is used by the procedural macros to derive Rust identifiers
/// from Diesel table/column names which typically follow `snake_case`. It
/// capitalizes the first letter of the resulting string and every letter that
/// follows an underscore.
///
/// Example: `"my_table_name"` -> `"MyTableName"`.
pub(crate) fn snake_to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Formats the provided iterator of tokenizable items as a nested tuple.
pub(crate) fn format_as_nested_tuple<I: DoubleEndedIterator<Item: ToTokens>>(
    items: I,
) -> proc_macro2::TokenStream {
    let mut rev_items = items.rev();
    if let Some(first) = rev_items.next() {
        rev_items.fold(quote::quote! { (#first,) }, |acc, item| {
            quote::quote! { (#item, #acc) }
        })
    } else {
        quote::quote! { () }
    }
}
