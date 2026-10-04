//! Native expansion shared by the `index!` and `unique_index!` compiler
//! adapters, unit tests, and fuzz targets.

/// Parsed representation of an index macro invocation.
struct IndexDefinition {
    /// The columns that form the index.
    columns: syn::punctuated::Punctuated<syn::Path, syn::Token![,]>,
}

impl syn::parse::Parse for IndexDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let columns = syn::punctuated::Punctuated::parse_terminated(input)?;
        Ok(IndexDefinition { columns })
    }
}

/// Expands an `index!` or `unique_index!` body into one marker impl per
/// column, rejecting bodies that are not a terminated list of column paths,
/// and rejecting a column whose output cannot itself parse as valid Rust (a
/// single reserved-keyword segment such as `try` parses as a `syn::Path`
/// but can never legally appear as an impl target).
pub fn expand_index(
    body: proc_macro2::TokenStream,
    trait_path: &proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    let index_def: IndexDefinition = syn::parse2(body)?;
    let columns: Vec<_> = index_def.columns.iter().map(|col| quote::quote! { #col }).collect();
    let impls = crate::utils::index_impls(trait_path, &columns);
    let expansion = quote::quote! { #(#impls)* };
    syn::parse2::<syn::File>(expansion.clone())
        .map_err(|error| syn::Error::new(proc_macro2::Span::call_site(), error))?;
    Ok(expansion)
}
