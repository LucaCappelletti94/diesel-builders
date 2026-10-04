//! Submodule providing the code generation for the `TakeColumn` trait
//! for the `NewValues` nested tuple in table builders.

/// Generate `TakeColumn` impls for each field in the struct.
pub(super) fn generate_take_column_impls(
    new_record_columns: &[syn::Path],
    table_module: &syn::Ident,
) -> proc_macro2::TokenStream {
    new_record_columns.iter().enumerate().map(|(idx, new_record_column)| {
		let typenum_index = crate::utils::typenum_ident(idx);
		let index_path = quote::quote! {
			::diesel_builders::typenum::#typenum_index
		};
        quote::quote! {
            impl ::diesel_builders::TakeColumn<#new_record_column> for <#table_module::table as ::diesel_builders::TableExt>::NewValues {
                #[inline]
                fn take_column(&mut self) -> Option<<#new_record_column as ::diesel_builders::ColumnTyped>::ColumnType> {
                    use ::diesel_builders::tuplities::NestedTupleIndexMut;
                    <Self as NestedTupleIndexMut<#index_path>>::nested_index_mut(self).take()
                }
            }
        }
    }).collect()
}
