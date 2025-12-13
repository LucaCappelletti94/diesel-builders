//! Procedural macros for diesel-builders workspace.
//!
//! This crate provides attribute macros that generate trait implementations
//! for tuples, replacing the complex `macro_rules!` patterns with cleaner
//! procedural macros.

/// Validation for const validators.
mod const_validator;
mod descendant;
/// Foreign primary key generation.
mod fpk;
mod table_model;
mod utils;
use proc_macro::TokenStream;

/// Derive macro to automatically implement `TypedColumn` for all table columns.
///
/// This macro should be derived on Model structs to automatically generate
/// `TypedColumn` implementations for each column based on the struct's field
/// types. It also automatically implements `GetColumn` for all fields, replacing
/// the need for a separate `GetColumn` derive.
///
/// Supports a helper attribute to override the insertable model name:
/// ```ignore
/// #[derive(TableModel)]
/// #[diesel(table_name = my_table)]
/// struct MyModel { ... }
/// ```
#[proc_macro_derive(
    TableModel,
    attributes(table_model, infallible, mandatory, discretionary, diesel)
)]
pub fn derive_table_model(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match table_model::derive_table_model_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Attribute macro to generate a const validator function from a `ValidateColumn` implementation.
///
/// This macro should be placed on `ValidateColumn` implementations that need compile-time
/// validation of default values. It generates a const function named `validate_{column_name}`
/// that can be used to validate default values at compile time.
///
/// # Example
///
/// ```ignore
/// #[const_validator]
/// impl ValidateColumn<animals::name> for AnimalsNewValues {
///     type Error = NewAnimalError;
///
///     fn validate_column(value: &String) -> Result<(), Self::Error> {
///         if value.is_empty() {
///             return Err(NewAnimalError::NameEmpty);
///         }
///         Ok(())
///     }
/// }
/// // Generates: pub const fn validate_name(value: &String) -> Result<(), NewAnimalError>
/// ```
#[proc_macro_attribute]
pub fn const_validator(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_impl = syn::parse_macro_input!(item as syn::ItemImpl);

    match const_validator::generate_const_validator(&mut item_impl) {
        Ok(const_fn) => {
            let output = quote::quote! {
                #item_impl
                #const_fn
            };
            output.into()
        }
        Err(err) => err.to_compile_error().into(),
    }
}

/// Define a foreign key relationship using SQL-like syntax.
///
/// This macro generates `HostColumn` implementations for each column in the foreign key.
/// The `ForeignKey` trait implementation is automatically provided by the `#[impl_foreign_key]`
/// procedural macro when all columns implement `HostColumn`.
#[proc_macro]
pub fn fk(input: TokenStream) -> TokenStream {
    use quote::quote;
    use syn::{
        parse::{Parse, ParseStream},
        punctuated::Punctuated,
        Token, Type,
    };

    /// Parsed representation of a `FOREIGN KEY` specification provided to
    /// the `fk!()` macro. `host_columns` are the columns on the local table,
    /// while `ref_columns` are the corresponding columns on the foreign table.
    struct ForeignKey {
        /// The host/source columns that form the foreign key on the local table.
        host_columns: Punctuated<Type, Token![,]>,
        /// The referenced/target columns that the host columns point to.
        ref_columns: Punctuated<Type, Token![,]>,
    }

    impl Parse for ForeignKey {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            // Parse: ( host_cols ) -> ( ref_cols )
            let host_content;
            syn::parenthesized!(host_content in input);
            let host_columns = Punctuated::parse_terminated(&host_content)?;

            input.parse::<Token![->]>()?;

            let ref_content;
            syn::parenthesized!(ref_content in input);
            let ref_columns = Punctuated::parse_terminated(&ref_content)?;

            Ok(ForeignKey {
                host_columns,
                ref_columns,
            })
        }
    }

    let fk_def = syn::parse_macro_input!(input as ForeignKey);
    let host_cols: Vec<_> = fk_def.host_columns.iter().collect();
    let ref_cols: Vec<_> = fk_def.ref_columns.iter().collect();

    if host_cols.len() != ref_cols.len() {
        return syn::Error::new_spanned(
            &fk_def.host_columns,
            "Number of host columns must match number of referenced columns",
        )
        .to_compile_error()
        .into();
    }

    // Generate HostColumn implementation for each column at its index
    let impls = host_cols.iter().enumerate().map(|(idx, host_col)| {
        let idx_type = syn::Ident::new(&format!("U{idx}"), proc_macro2::Span::call_site());
        quote! {
            impl diesel_builders::HostColumn<
                diesel_builders::typenum::#idx_type,
                ( #(#host_cols,)* ),
                ( #(#ref_cols,)* )
            > for #host_col {}
        }
    });

    quote! {
        #(#impls)*
    }
    .into()
}

/// Define a foreign primary key relationship using SQL-like syntax.
///
/// This macro generates a `ForeignPrimaryKey` implementation for a single column
/// that references a table's primary key, along with a helper trait method to
/// fetch the foreign record.
///
/// # Syntax
///
/// ```ignore
/// fpk!(column_path -> referenced_table);
/// ```
///
/// # Example
///
/// ```ignore
/// fpk!(discretionary_table::parent_id -> parent_table);
/// ```
#[proc_macro]
pub fn fpk(input: TokenStream) -> TokenStream {
    use syn::{
        parse::{Parse, ParseStream},
        Path, Token,
    };

    /// Parsed representation of a foreign primary key specification.
    struct ForeignPrimaryKey {
        /// The column that is the foreign key.
        column: Path,
        /// The referenced table.
        referenced_table: Path,
    }

    impl Parse for ForeignPrimaryKey {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            // Parse: column_path -> referenced_table
            let column: Path = input.parse()?;
            input.parse::<Token![->]>()?;
            let referenced_table: Path = input.parse()?;

            Ok(ForeignPrimaryKey {
                column,
                referenced_table,
            })
        }
    }

    let fpk_def = syn::parse_macro_input!(input as ForeignPrimaryKey);
    let column = &fpk_def.column;
    let referenced_table = &fpk_def.referenced_table;
    let referenced_type: syn::Type = syn::parse_quote!(#referenced_table);

    fpk::generate_fpk_impl(column, &referenced_type).into()
}

/// Define a table index using SQL-like syntax.
///
/// This macro generates `IndexedColumn` implementations for each column in the index.
/// The `TableIndex` trait implementation is automatically provided by the `#[impl_table_index]`
/// procedural macro when all columns implement `IndexedColumn`.
#[proc_macro]
pub fn index(input: TokenStream) -> TokenStream {
    use quote::quote;
    use syn::{
        parse::{Parse, ParseStream},
        punctuated::Punctuated,
        Token, Type,
    };

    /// Parsed representation of an `INDEX` macro invocation, containing the
    /// columns that form the index definition.
    struct TableIndex {
        /// The columns included in the index.
        columns: Punctuated<Type, Token![,]>,
    }

    impl Parse for TableIndex {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            // Accept either: a parenthesized tuple `(col1, col2)` or a bare list `col1, col2`.
            if input.peek(syn::token::Paren) {
                let content;
                syn::parenthesized!(content in input);
                let columns = Punctuated::parse_terminated(&content)?;
                Ok(TableIndex { columns })
            } else {
                let columns = Punctuated::parse_terminated(input)?;
                Ok(TableIndex { columns })
            }
        }
    }

    let index_def = syn::parse_macro_input!(input as TableIndex);
    let cols: Vec<_> = index_def.columns.iter().collect();

    // Generate IndexedColumn implementation for each column at its index
    let impls = cols.iter().enumerate().map(|(idx, col)| {
        let idx_type = syn::Ident::new(&format!("U{idx}"), proc_macro2::Span::call_site());
        quote! {
            impl diesel_builders::IndexedColumn<
                diesel_builders::typenum::#idx_type,
                ( #(#cols,)* )
            > for #col {}
        }
    });

    quote! {
        #(#impls)*
    }
    .into()
}
