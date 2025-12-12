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

/// Macro to declare a singleton foreign key relationship.
///
/// This macro:
/// 1. Implements `ForeignPrimaryKey` for a column that references another table's primary key
/// 2. Generates a helper trait with a method to fetch the foreign record using `GetForeignExt`
///
/// # Method naming
/// - If column name ends with `_id` (e.g., `a_id`), the method will be named `a`
/// - Otherwise, the method will be named `{column_name}_fk`
///
/// # Example
/// ```ignore
/// fpk!(table_b::c_id -> table_c);
/// ```
///
/// This generates:
/// - `impl ForeignPrimaryKey for table_b::c_id { type ReferencedTable = table_c::table; }`
/// - A trait `GetTableBC` with method `c(&self, conn: &mut Conn)` that calls `get_foreign`
#[proc_macro]
#[allow(clippy::too_many_lines)]
pub fn fpk(input: TokenStream) -> TokenStream {
    use quote::quote;
    use syn::{
        parse::{Parse, ParseStream},
        Path, Token,
    };

    /// Parsed representation of a singleton foreign key declaration.
    struct ForeignPrimaryKeyDecl {
        /// The column that is the foreign key (e.g., `table_b::c_id`)
        column: Path,
        /// The referenced table (e.g., `table_c`)
        referenced_table: Path,
    }

    impl Parse for ForeignPrimaryKeyDecl {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let column: Path = input.parse()?;
            input.parse::<Token![->]>()?;
            let referenced_table: Path = input.parse()?;
            Ok(ForeignPrimaryKeyDecl {
                column,
                referenced_table,
            })
        }
    }

    let decl = syn::parse_macro_input!(input as ForeignPrimaryKeyDecl);
    let column = decl.column;
    let referenced_table = decl.referenced_table;

    // Extract column name for method generation — must be present
    let column_name = match column.segments.last() {
        Some(seg) => seg.ident.to_string(),
        None => {
            return syn::Error::new_spanned(
                &column,
                "fpk! macro requires a column path (e.g., `table_b::c_id`). Could not extract column name.",
            )
            .to_compile_error()
            .into();
        }
    };

    // Extract referenced table name for method generation — must be present
    let referenced_table_name = match referenced_table.segments.last() {
        Some(seg) => seg.ident.to_string(),
        None => {
            // If we cannot extract a referenced table name, fail the macro with a helpful message
            return syn::Error::new_spanned(
                &referenced_table,
                "fpk! macro requires a referenced table path (e.g., `table_c`). Could not extract referenced table name.",
            )
            .to_compile_error()
            .into();
        }
    };

    // Generate method name based on column name
    let method_name = if let Some(stripped) = column_name.strip_suffix("_id") {
        stripped.to_string()
    } else {
        format!("{column_name}_fk")
    };
    let method_ident = syn::Ident::new(&method_name, proc_macro2::Span::call_site());

    // Generate trait name
    // Extract table name from column path (second-to-last segment)
    let table_name_segment = if column.segments.len() >= 2 {
        column.segments[column.segments.len() - 2].ident.to_string()
    } else {
        "table".to_string()
    };

    // Convert table_name to CamelCase for trait name
    let trait_name = format!(
        "FK{}{}",
        crate::utils::snake_to_camel_case(&table_name_segment),
        crate::utils::snake_to_camel_case(&column_name)
    );
    let trait_ident = syn::Ident::new(&trait_name, proc_macro2::Span::call_site());

    // Generate documentation
    let trait_doc = format!("Trait to get the foreign record referenced by `{column_name}`.");
    let method_doc = format!(
        "Fetches the foreign `{referenced_table_name}` record referenced by this `{column_name}`."
    );

    quote! {
        impl diesel_builders::ForeignPrimaryKey for #column {
            type ReferencedTable = #referenced_table::table;
        }

        #[doc = #trait_doc]
        pub trait #trait_ident<Conn>: diesel_builders::GetForeignExt<Conn> {
            #[doc = #method_doc]
            #[doc = ""]
            #[doc = "# Arguments"]
            #[doc = ""]
            #[doc = "* `conn` - A mutable reference to the database connection."]
            #[doc = ""]
            #[doc = "# Errors"]
            #[doc = "Returns a `diesel::QueryResult` error if the query fails or no matching record is found."]
            #[inline]
            fn #method_ident(
                &self,
                conn: &mut Conn,
            ) -> diesel::QueryResult<<#referenced_table::table as diesel_builders::TableExt>::Model>
            where
                Self: diesel_builders::GetForeign<
                    Conn,
                    (#column,),
                    (<#referenced_table::table as diesel::Table>::PrimaryKey,),
                >,
            {
                <Self as diesel_builders::GetForeign<
                    Conn,
                    (#column,),
                    (<#referenced_table::table as diesel::Table>::PrimaryKey,),
                >>::get_foreign(self, conn)
            }
        }

        impl<T, Conn> #trait_ident<Conn> for T
        where
            T: diesel_builders::GetForeign<
                Conn,
                (#column,),
                (<#referenced_table::table as diesel::Table>::PrimaryKey,)
            > {}
    }
    .into()
}
