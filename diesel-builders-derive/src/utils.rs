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
    let mut result = String::with_capacity(s.len());
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
pub(crate) fn format_as_nested_tuple<
    I: IntoIterator<Item: ToTokens, IntoIter: DoubleEndedIterator>,
>(
    items: I,
) -> proc_macro2::TokenStream {
    let mut rev_items = items.into_iter().rev();
    if let Some(first) = rev_items.next() {
        rev_items.fold(quote::quote! { (#first,) }, |acc, item| {
            quote::quote! { (#item, #acc) }
        })
    } else {
        quote::quote! { () }
    }
}

/// Checks if the given type is an `Option`.
pub(crate) fn is_option(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Option";
    }
    false
}

/// Convert a `CamelCase` string to `snake_case`.
pub(crate) fn camel_to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}
