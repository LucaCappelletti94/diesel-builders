//! Submodule providing the code generation for the `TrySetColumn` trait
//! for the `NewValues` nested tuple in table builders.

/// Generate `TrySetColumn` impls for each field in the struct.
pub fn generate_set_column_impls(
    new_record_columns: &[(usize, syn::Path)],
    table_module: &syn::Ident,
) -> proc_macro2::TokenStream {
    new_record_columns.iter().map(|(idx, new_record_column)| {
		let typenum_index = syn::Ident::new(&format!("U{idx}"), proc_macro2::Span::call_site());
		let index_path = quote::quote! {
			diesel_builders::typenum::#typenum_index
		};
        quote::quote! {
            impl diesel_builders::TrySetColumn<#new_record_column> for <#table_module::table as diesel_builders::TableExt>::NewValues {
                type Error = core::convert::Infallible;

                #[inline]
                fn try_set_column(&mut self, value: <#new_record_column as diesel_builders::Typed>::Type) -> Result<&mut Self, Self::Error> {
                    use diesel_builders::tuplities::NestedTupleIndexMut;
                    *<Self as NestedTupleIndexMut<#index_path>>::nested_index_mut(self) = Some(value);
                    Ok(self)
                }
            }
        }
    }).collect()
}

/// Generate implementations of `SetColumnUnchecked` for each field in the struct.
pub fn generate_set_column_unchecked_traits(
    new_record_columns: &[(usize, syn::Path)],
    table_module: &syn::Ident,
) -> proc_macro2::TokenStream {
    new_record_columns.iter().map(|(idx, new_record_column)| {
		let typenum_index = syn::Ident::new(&format!("U{idx}"), proc_macro2::Span::call_site());
		let index_path = quote::quote! {
			diesel_builders::typenum::#typenum_index
		};
        quote::quote! {
            impl diesel_builders::SetColumnUnchecked<#new_record_column> for <#table_module::table as diesel_builders::TableExt>::NewValues {
                #[inline]
                fn set_column_unchecked(&mut self, value: impl Into<<#new_record_column as diesel_builders::Typed>::Type>) -> &mut Self {
                    use diesel_builders::tuplities::NestedTupleIndexMut;
                    *<Self as NestedTupleIndexMut<#index_path>>::nested_index_mut(self) = Some(value.into());
                    self
                }
            }
        }
    }).collect()
}
