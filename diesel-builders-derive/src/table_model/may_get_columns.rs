//! Submodule providing the code generation for the `MayGetColumn` trait
//! for the `NewValues` nested tuple in table builders.

/// Generate `MayGetColumn` impls for each field in the struct.
pub fn generate_may_get_column_impls(
    new_record_columns: &[syn::Path],
    table_module: &syn::Ident,
) -> proc_macro2::TokenStream {
    new_record_columns.iter().enumerate().map(|(idx, new_record_column)| {
		let typenum_index = syn::Ident::new(&format!("U{idx}"), proc_macro2::Span::call_site());
		let index_path = quote::quote! {
			::diesel_builders::typenum::#typenum_index
		};
        quote::quote! {
            impl ::diesel_builders::MayGetColumn<#new_record_column> for <#table_module::table as ::diesel_builders::TableExt>::NewValues {
            fn may_get_column_ref<'a>(&'a self) -> Option<&'a <#new_record_column as ::diesel_builders::Typed>::ColumnType>
                    where
                        #table_module::table: 'a,
                {
					use ::diesel_builders::tuplities::NestedTupleIndex;
                    <Self as NestedTupleIndex<#index_path>>::nested_index(self).as_ref()
                }
            }
        }
    }).collect()
}
