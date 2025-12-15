//! Foreign primary key generation utilities.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Path, Type};

/// Generate a foreign primary key implementation for a column.
///
/// This function generates:
/// 1. `ForeignPrimaryKey` implementation for the column
/// 2. A helper trait with a method to fetch the foreign record
///
/// # Arguments
/// * `column` - The column path (e.g., `table_b::c_id`)
/// * `referenced_table` - The referenced table type (e.g., `table_c`)
pub fn generate_fpk_impl(column: &Path, referenced_table: &Type) -> TokenStream {
    // Extract the path from the Type if it's a path type
    let referenced_table_path = match referenced_table {
        Type::Path(type_path) => &type_path.path,
        _ => {
            // If it's not a path type, we can't extract the information needed
            // Return an empty token stream
            return quote! {};
        }
    };

    // Extract column name for method generation
    let column_name = column
        .segments
        .last()
        .expect("Column path must have at least one segment")
        .ident
        .to_string();

    // Extract referenced table name for method generation
    let referenced_table_name = referenced_table_path
        .segments
        .last()
        .expect("Referenced table path must have at least one segment")
        .ident
        .to_string();

    // Generate method name based on column name
    let method_name = if let Some(stripped) = column_name.strip_suffix("_id") {
        stripped.to_string()
    } else {
        format!("{column_name}_fk")
    };
    let method_ident = syn::Ident::new(&method_name, proc_macro2::Span::call_site());

    // Generate trait name
    // Extract table name from column path (second-to-last segment)
    assert!(
        column.segments.len() >= 2,
        "Column path must have at least 2 segments (table::column)"
    );
    let table_name_segment = column.segments[column.segments.len() - 2].ident.to_string();

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
                >>::foreign(self, conn)
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
}

/// Generate a foreign primary key implementation for a column with a Path reference.
///
/// This is a convenience wrapper for `generate_fpk_impl` that converts a Path to Type.
pub fn generate_fpk_impl_from_paths(column: &Path, referenced_table: &Path) -> TokenStream {
    let referenced_type: Type = syn::parse_quote!(#referenced_table);
    generate_fpk_impl(column, &referenced_type)
}
