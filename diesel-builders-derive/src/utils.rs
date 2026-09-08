//! Submodule with utilities for the diesel-builders macros.

use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
    sync::{Mutex, OnceLock},
};

use quote::ToTokens;

/// Builds the `typenum` unsigned marker identifier `U{index}` such as `U0`.
pub(crate) fn typenum_ident(index: usize) -> syn::Ident {
    syn::Ident::new(&format!("U{index}"), proc_macro2::Span::call_site())
}

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
    let mut items = items.into_iter();
    let Some(last) = items.next_back() else {
        return quote::quote! { () };
    };

    items.rev().fold(quote::quote! { (#last,) }, |acc, item| {
        quote::quote! { (#item, #acc) }
    })
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

/// Returns the first generic type argument of a type path, such as the `T` in
/// `Foo<T, ..>`, or `None` when the type has no angle-bracketed type argument.
pub(crate) fn first_generic_arg(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(type_path) = ty else {
        return None;
    };
    let syn::PathArguments::AngleBracketed(args) = &type_path.path.segments.last()?.arguments
    else {
        return None;
    };
    match args.args.first()? {
        syn::GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}

/// Returns the inner type `T` of an `Option<T>`, or `None` for any other type.
pub(crate) fn option_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    is_option(ty).then(|| first_generic_arg(ty)).flatten()
}

/// Emits an index-marker `impl` for every column of an index.
///
/// Each column `col` at position `idx` gets
/// `impl #trait_path<typenum::U{idx}, ( #columns, )> for col {}`, shared by the
/// primary-key codegen and the `index!` / `unique_index!` proc macros.
pub(crate) fn index_impls(
    trait_path: &proc_macro2::TokenStream,
    columns: &[proc_macro2::TokenStream],
) -> Vec<proc_macro2::TokenStream> {
    columns
        .iter()
        .enumerate()
        .map(|(idx, col)| {
            let idx_type = typenum_ident(idx);
            quote::quote! {
                impl #trait_path<
                    ::diesel_builders::typenum::#idx_type,
                    ( #(#columns,)* )
                > for #col {}
            }
        })
        .collect()
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

/// Helper to determine if we should generate
/// `allow_tables_to_appear_in_same_query`.
///
/// Returns `true` if this pair hasn't been generated yet.
/// Uses a static lookup struct to track pairs.
fn should_generate_allow_tables_to_appear_in_same_query(t1: &syn::Path, t2: &syn::Path) -> bool {
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

/// Emits a `diesel::allow_tables_to_appear_in_same_query!` invocation for the
/// pair of tables, or `None` when the pair has already been generated.
///
/// Wraps `should_generate_allow_tables_to_appear_in_same_query` so callers
/// declare a joinable pair in one expression instead of repeating the guard and
/// the macro call.
pub(crate) fn allow_tables_to_appear_in_same_query(
    t1: &syn::Path,
    t2: &syn::Path,
) -> Option<proc_macro2::TokenStream> {
    should_generate_allow_tables_to_appear_in_same_query(t1, t2).then(|| {
        quote::quote! {
            ::diesel::allow_tables_to_appear_in_same_query!(#t1, #t2);
        }
    })
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
