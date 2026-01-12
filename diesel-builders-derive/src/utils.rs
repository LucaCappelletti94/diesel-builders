//! Submodule with utilities for the diesel-builders macros.

use quote::ToTokens;
use std::collections::HashSet;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::{Mutex, OnceLock};

/// Static lookup struct to track which table pairs have already had
/// `diesel::allow_tables_to_appear_in_same_query!` generated.
/// This prevents duplicate macro invocations which would cause compile errors.
static GENERATED_LINKS: OnceLock<Mutex<HashSet<u64>>> = OnceLock::new();

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

/// Helper to determine if we should generate `allow_tables_to_appear_in_same_query`.
///
/// Returns `true` if this pair hasn't been generated yet.
/// Uses a static lookup struct to track pairs.
pub(crate) fn should_generate_allow_tables_to_appear_in_same_query(
    t1: &syn::Path,
    t2: &syn::Path,
) -> bool {
    // Initialize the static map if needed
    let map = GENERATED_LINKS.get_or_init(|| Mutex::new(HashSet::new()));

    let Some(s1) = t1.segments.last().map(|seg| &seg.ident) else {
        return false;
    };
    let Some(s2) = t2.segments.last().map(|seg| &seg.ident) else {
        return false;
    };

    // Same table, no need to generate
    if s1 == s2 {
        return false;
    }

    // Sort to handle symmetry (A, B) == (B, A)
    let pair = if s1 < s2 { (s1, s2) } else { (s2, s1) };

    let hash = {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        pair.hash(&mut hasher);
        hasher.finish()
    };

    let mut lock = map.lock().unwrap();
    lock.insert(hash)
}

/// Extracts the table path from a column path.
/// Assumes standard Diesel format `Module::Table::Column`.
/// Returns the path without the last segment.
pub(crate) fn extract_table_path_from_column(path: &syn::Path) -> Option<syn::Path> {
    if path.segments.len() < 2 {
        return None;
    }
    let mut table_path = path.clone();
    table_path.segments.pop();
    table_path.segments.pop_punct();
    Some(table_path)
}
