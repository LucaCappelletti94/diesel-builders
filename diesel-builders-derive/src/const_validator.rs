use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemImpl, parse_quote};

/// Generate a const validator function from a `ValidateColumn` implementation.
///
/// This macro extracts the validation logic from the `validate_column` method,
/// moves it into a standalone `const fn validate_{column_name}`, and replaces
/// the method body to call that const function. This enables compile-time
/// validation of default values while keeping the trait method non-const.
#[allow(clippy::too_many_lines)]
pub fn generate_const_validator(item: &mut ItemImpl) -> syn::Result<TokenStream> {
    // Extract the trait path and column type
    let trait_impl = item
        .trait_
        .as_ref()
        .ok_or_else(|| syn::Error::new_spanned(&*item, "Expected trait implementation"))?;

    let trait_path = &trait_impl.1;

    // Verify this is a ValidateColumn implementation
    let trait_name = quote!(#trait_path).to_string();
    if !trait_name.contains("ValidateColumn") {
        return Err(syn::Error::new_spanned(
            trait_path,
            "This macro can only be applied to `ValidateColumn` implementations",
        ));
    }

    // Extract the column type from the generics
    // The trait should be ValidateColumn<ColumnType>
    let syn::Path { segments, .. } = trait_path;
    let column_type = {
        let last_segment = segments
            .last()
            .ok_or_else(|| syn::Error::new_spanned(trait_path, "Expected trait path"))?;

        match &last_segment.arguments {
            syn::PathArguments::AngleBracketed(args) => {
                let first_arg = args.args.first().ok_or_else(|| {
                    syn::Error::new_spanned(
                        args,
                        "ValidateColumn must have a column type parameter",
                    )
                })?;

                match first_arg {
                    syn::GenericArgument::Type(ty) => ty,
                    _ => return Err(syn::Error::new_spanned(first_arg, "Expected type argument")),
                }
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    last_segment,
                    "ValidateColumn must have angle-bracketed type parameters",
                ));
            }
        }
    };

    // Extract column name from the column type (e.g., animals::name -> name)
    let column_name = extract_column_name(column_type)?;

    // Extract the error type from the associated type (before mutably borrowing items)
    let error_type = item
        .items
        .iter()
        .find_map(|impl_item| {
            if let syn::ImplItem::Type(type_item) = impl_item {
                if type_item.ident == "Error" {
                    Some(type_item.ty.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .ok_or_else(|| {
            syn::Error::new_spanned(&*item, "`ValidateColumn` must define `Error` type")
        })?;

    // Extract the Borrowed type from the associated type
    let borrowed_type = item
        .items
        .iter()
        .find_map(|impl_item| {
            if let syn::ImplItem::Type(type_item) = impl_item {
                if type_item.ident == "Borrowed" {
                    Some(type_item.ty.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .ok_or_else(|| {
            syn::Error::new_spanned(&*item, "`ValidateColumn` must define `Borrowed` type")
        })?;

    // Store the self_ty for error messages (before mutably borrowing items)
    let impl_self_ty = item.self_ty.clone();

    // Find and extract the validate_column method
    let validate_method = item
        .items
        .iter_mut()
        .find_map(|impl_item| {
            if let syn::ImplItem::Fn(method) = impl_item {
                if method.sig.ident == "validate_column" {
                    Some(method)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .ok_or_else(|| {
            syn::Error::new_spanned(
                &impl_self_ty,
                "`ValidateColumn` implementation must have a `validate_column` method",
            )
        })?;

    // Extract the value parameter name
    let value_param = validate_method
        .sig
        .inputs
        .iter()
        .next() // First parameter (the static method has no self)
        .ok_or_else(|| {
            syn::Error::new_spanned(
                &validate_method.sig,
                "`validate_column` must have a `value` parameter",
            )
        })?;

    let value_param_name = match value_param {
        syn::FnArg::Typed(pat_type) => match &*pat_type.pat {
            syn::Pat::Ident(ident) => ident.ident.clone(),
            _ => {
                return Err(syn::Error::new_spanned(
                    &pat_type.pat,
                    "Expected simple parameter name",
                ));
            }
        },
        syn::FnArg::Receiver(_) => {
            return Err(syn::Error::new_spanned(
                value_param,
                "Expected typed parameter",
            ));
        }
    };

    // Extract the original method body
    let original_body = validate_method.block.clone();

    // Generate the const validator function name
    let validator_name = syn::Ident::new(&format!("validate_{column_name}"), column_name.span());

    // Generate the const validator function with the extracted logic
    let const_validator = quote! {
        #[doc = concat!("Const validator for ", stringify!(#column_type), " default values.")]
        #[doc = ""]
        #[doc = "This function is automatically generated by the `const_validator` attribute."]
        #[doc = "It validates default values at compile time."]
        #[doc = ""]
        #[doc = "If this function fails to compile as `const fn`, it means the validation logic"]
        #[doc = "uses non-const operations. Consider simplifying the validation or removing the"]
        #[doc = "`const_validator` attribute if compile-time validation is not needed."]
        pub const fn #validator_name(#value_param_name: &#borrowed_type) -> Result<(), #error_type> {
            #original_body
        }
    };

    // Replace the method body to call the const function
    validate_method.block = parse_quote! {
        {
            #validator_name(#value_param_name)
        }
    };

    Ok(const_validator)
}

/// Extract the column name from a column type path.
///
/// For example, `animals::name` becomes `name`.
fn extract_column_name(column_type: &syn::Type) -> syn::Result<syn::Ident> {
    match column_type {
        syn::Type::Path(type_path) => {
            let last_segment = type_path.path.segments.last().ok_or_else(|| {
                syn::Error::new_spanned(column_type, "Expected column path with segments")
            })?;

            Ok(last_segment.ident.clone())
        }
        _ => Err(syn::Error::new_spanned(
            column_type,
            "Expected column type to be a path (e.g., table::column)",
        )),
    }
}
