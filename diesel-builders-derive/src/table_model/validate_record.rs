//! Submodule providing the code generation for the infallible
//! `ValidateRecord` implementation of tables that declare no record error
//! type.

/// Generate the infallible `ValidateRecord` impl for a table's `NewValues`.
pub(super) fn generate_infallible_validate_record_impl(
    table_module: &syn::Ident,
) -> proc_macro2::TokenStream {
    quote::quote! {
        impl ::diesel_builders::ValidateRecord<#table_module::table> for <#table_module::table as ::diesel_builders::TableExt>::NewValues {
            fn validate_record(
                &self,
            ) -> core::result::Result<
                (),
                <#table_module::table as ::diesel_builders::TableExt>::RecordError,
            > {
                Ok(())
            }
        }
    }
}
