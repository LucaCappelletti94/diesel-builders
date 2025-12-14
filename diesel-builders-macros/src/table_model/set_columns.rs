//! Submodule providing the code generation for the `SetColumn` trait
//! for the `NewValues` nested tuple in table builders.

/// Generate `SetColumn` impls for each field in the struct.
pub(super) fn generate_set_column_impls(
    new_record_columns: &[syn::Path],
    table_module: &syn::Ident,
) -> proc_macro2::TokenStream {
    new_record_columns.iter().enumerate().map(|(idx, new_record_column)| {
		let typenum_index = syn::Ident::new(&format!("U{idx}"), proc_macro2::Span::call_site());
		let index_path = quote::quote! {
			diesel_builders::typenum::#typenum_index
		};
        quote::quote! {
            impl diesel_builders::SetColumn<#new_record_column> for <#table_module::table as diesel_builders::TableExt>::NewValues {
                #[inline]
                fn set_column(&mut self, value: impl Into<<#new_record_column as diesel_builders::Typed>::ColumnType>) -> &mut Self {
                    use diesel_builders::tuplities::NestedTupleIndexMut;
                    *<Self as NestedTupleIndexMut<#index_path>>::nested_index_mut(self) = Some(value.into());
                    self
                }
            }
        }
    }).collect()
}

/// Generate `ValidateColumn` implementations for infallible records.
pub(super) fn generate_infallible_validate_column_impls(
    infallible_records: &[syn::Path],
    table_module: &syn::Ident,
) -> proc_macro2::TokenStream {
    infallible_records.iter().map(|infallible_record| {
        quote::quote! {
            impl diesel_builders::ValidateColumn<#infallible_record> for <#table_module::table as diesel_builders::TableExt>::NewValues {
                type Error = core::convert::Infallible;

                #[inline]
                fn validate_column(value: &<#infallible_record as diesel_builders::Typed>::ColumnType) -> Result<(), Self::Error> {
                    Ok(())
                }
            }
        }
    }).collect()
}
