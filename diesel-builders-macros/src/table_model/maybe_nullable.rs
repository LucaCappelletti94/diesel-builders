use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Field, Path, Type};

/// Check if a type is an `Option`.
fn is_option(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

/// Generate `MaybeNullable` implementations for columns.
pub fn generate_maybe_nullable_impls(
    fields: &Punctuated<Field, Comma>,
    table_path: &Path,
) -> TokenStream {
    let mut impls = TokenStream::new();

    for field in fields {
        if let Some(field_name) = &field.ident {
            let column_path = quote! { #table_path::#field_name };
            let field_ty = &field.ty;

            if is_option(field_ty) {
                impls.extend(quote! {
                    impl diesel_builders::columns::MaybeNullable for #column_path {
                        type Output = diesel::dsl::Nullable<#column_path>;
                        fn maybe_nullable(self) -> Self::Output {
                            use diesel::NullableExpressionMethods;
                            self.nullable()
                        }
                    }
                });
            } else {
                impls.extend(quote! {
                    impl diesel_builders::columns::MaybeNullable for #column_path {
                        type Output = #column_path;
                        fn maybe_nullable(self) -> Self::Output {
                            self
                        }
                    }
                });
            }
        }
    }

    impls
}
