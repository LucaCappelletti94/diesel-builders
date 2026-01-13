//! Procedural macros for diesel-builders workspace.
//!
//! This crate provides attribute macros that generate trait implementations
//! for tuples, replacing the complex `macro_rules!` patterns with cleaner
//! procedural macros.

mod descendant;
mod table_model;
mod utils;
use proc_macro::TokenStream;
/// Derive macro to automatically implement `TypedColumn` for all table columns.
///
/// This macro should be derived on Model structs to automatically generate
/// `TypedColumn` implementations for each column based on the struct's field
/// types. It also automatically implements `GetColumn` for all fields,
/// replacing the need for a separate `GetColumn` derive.
///
/// Supports a helper attribute to override the insertable model name:
/// ```ignore
/// #[derive(TableModel)]
/// #[diesel(table_name = my_table)]
/// struct MyModel { ... }
/// ```
#[proc_macro_derive(
    TableModel,
    attributes(table_model, infallible, mandatory, discretionary, diesel, same_as)
)]
pub fn derive_table_model(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match table_model::derive_table_model_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Parsed representation of an index macro invocation.
struct IndexDefinition {
    /// The columns that form the index.
    columns: syn::punctuated::Punctuated<syn::Type, syn::Token![,]>,
}

impl syn::parse::Parse for IndexDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let columns = syn::punctuated::Punctuated::parse_terminated(input)?;
        Ok(IndexDefinition { columns })
    }
}

/// Helper function to generate index implementations for each column in the
/// index.
fn generate_index_impl(input: TokenStream, trait_path: &proc_macro2::TokenStream) -> TokenStream {
    let index_def = syn::parse_macro_input!(input as IndexDefinition);
    let cols: Vec<_> = index_def.columns.iter().collect();

    let impls = cols.iter().enumerate().map(|(idx, col)| {
        let idx_type = syn::Ident::new(&format!("U{idx}"), proc_macro2::Span::call_site());
        quote::quote! {
            impl #trait_path<
                diesel_builders::typenum::#idx_type,
                ( #(#cols,)* )
            > for #col {}
        }
    });

    quote::quote! {
        #(#impls)*
    }
    .into()
}

/// Define a table UNIQUE index using SQL-like syntax.
///
/// This macro generates `UniquelyIndexedColumn` implementations for each column
/// in the index.
#[proc_macro]
pub fn unique_index(input: TokenStream) -> TokenStream {
    generate_index_impl(input, &quote::quote!(diesel_builders::UniquelyIndexedColumn))
}

/// Define a table index using SQL-like syntax.
///
/// This macro generates `IndexedColumn` implementations for each column in the
/// index.
#[proc_macro]
pub fn index(input: TokenStream) -> TokenStream {
    generate_index_impl(input, &quote::quote!(diesel_builders::IndexedColumn))
}
