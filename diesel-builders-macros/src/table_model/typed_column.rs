//! `TypedColumn` trait implementations and associated setter/getter traits.

use crate::utils::snake_to_camel_case;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, Field, Ident, Token};

/// Generate `TypedColumn` implementations and associated setter/getter traits for all fields.
pub fn generate_typed_column_impls(
    fields: &Punctuated<Field, Token![,]>,
    table_module: &syn::Ident,
    struct_ident: &Ident,
    primary_key_columns: &[Ident],
) -> TokenStream {
    fields
        .iter()
        .filter_map(|field| {
            let field_name = field.ident.as_ref()?;
            let field_type = &field.ty;

            Some(generate_field_traits(
                field,
                field_name,
                field_type,
                table_module,
                struct_ident,
                primary_key_columns,
            ))
        })
        .collect()
}

/// Generate all trait implementations for a single field.
fn generate_field_traits(
    field: &Field,
    field_name: &Ident,
    field_type: &syn::Type,
    table_module: &syn::Ident,
    struct_ident: &Ident,
    primary_key_columns: &[Ident],
) -> TokenStream {
    use crate::table_model::attribute_parsing::{is_field_discretionary, is_field_mandatory};

    let camel_cased_field_name = snake_to_camel_case(&field_name.to_string());

    // Generate getter trait only for non-id fields
    let maybe_getter_impl = (field_name != "id").then(|| {
        generate_getter_trait(
            field_name,
            table_module,
            struct_ident,
            &camel_cased_field_name,
        )
    });

    // Determine triangular relation type
    let is_mandatory = is_field_mandatory(field);
    let is_discretionary = is_field_discretionary(field);

    // Generate triangular relation traits only for single primary key tables and if field is marked
    let maybe_triangular_impls =
        if primary_key_columns.len() == 1 && (is_mandatory || is_discretionary) {
            Some(generate_triangular_relation_traits(
                field_name,
                table_module,
                struct_ident,
                &camel_cased_field_name,
                is_mandatory,
                is_discretionary,
            ))
        } else {
            None
        };

    let set_trait = generate_set_trait(
        field_name,
        table_module,
        struct_ident,
        &camel_cased_field_name,
    );
    let try_set_trait = generate_try_set_trait(
        field_name,
        table_module,
        struct_ident,
        &camel_cased_field_name,
    );
    let typed_impl = generate_typed_impl(field_name, field_type, table_module);

    quote! {
        #maybe_getter_impl
        #maybe_triangular_impls
        #set_trait
        #try_set_trait
        #typed_impl
    }
}

/// Generate the getter trait for a field.
fn generate_getter_trait(
    field_name: &Ident,
    table_module: &syn::Ident,
    struct_ident: &Ident,
    camel_cased_field_name: &str,
) -> TokenStream {
    let get_field_name = syn::Ident::new(
        &format!("Get{struct_ident}{camel_cased_field_name}"),
        proc_macro2::Span::call_site(),
    );

    let get_trait_doc_comment =
        format!("Trait to get the `{field_name}` column from a `{table_module}` table model.");
    let get_field_name_method_doc_comment =
        format!("Gets the value of the `{field_name}` column from a `{table_module}` table model.");

    quote! {
        #[doc = #get_trait_doc_comment]
        pub trait #get_field_name: diesel_builders::GetColumn<#table_module::#field_name> {
            #[inline]
            #[doc = #get_field_name_method_doc_comment]
            fn #field_name(&self) -> &<#table_module::#field_name as diesel_builders::Typed>::ColumnType {
                self.get_column_ref()
            }
        }
        impl<T> #get_field_name for T where T: diesel_builders::GetColumn<#table_module::#field_name> {}
    }
}

/// Generate the `SetColumn` trait for a field.
fn generate_set_trait(
    field_name: &Ident,
    table_module: &syn::Ident,
    struct_ident: &Ident,
    camel_cased_field_name: &str,
) -> TokenStream {
    let set_field_name = syn::Ident::new(
        &format!("Set{struct_ident}{camel_cased_field_name}"),
        proc_macro2::Span::call_site(),
    );
    let field_name_ref =
        syn::Ident::new(&format!("{field_name}_ref"), proc_macro2::Span::call_site());

    let set_trait_doc_comment =
        format!("Trait to set the `{field_name}` column on a [`{table_module}`] table builder.");
    let field_name_ref_method_doc_comment = format!(
        "Sets the `{field_name}` column on a [`{table_module}`] table builder by reference."
    );
    let field_name_method_doc_comment =
        format!("Sets the `{field_name}` column on a [`{table_module}`] table builder.");

    quote! {
        #[doc = #set_trait_doc_comment]
        pub trait #set_field_name: diesel_builders::SetColumn<#table_module::#field_name> + Sized {
            #[inline]
            #[doc = #field_name_ref_method_doc_comment]
            fn #field_name_ref(
                &mut self,
                value: impl Into<<#table_module::#field_name as diesel_builders::Typed>::ColumnType>
            ) -> &mut Self {
                use diesel_builders::SetColumnExt;
                self.set_column_ref::<#table_module::#field_name>(value)
            }
            #[inline]
            #[must_use]
            #[doc = #field_name_method_doc_comment]
            fn #field_name(
                self,
                value: impl Into<<#table_module::#field_name as diesel_builders::Typed>::ColumnType>
            ) -> Self {
                use diesel_builders::SetColumnExt;
                self.set_column::<#table_module::#field_name>(value)
            }
        }

        impl<T> #set_field_name for T where T: diesel_builders::SetColumn<#table_module::#field_name> {}
    }
}

/// Generate the `TrySetColumn` trait for a field.
fn generate_try_set_trait(
    field_name: &Ident,
    table_module: &syn::Ident,
    struct_ident: &Ident,
    camel_cased_field_name: &str,
) -> TokenStream {
    let try_set_field_name = syn::Ident::new(
        &format!("TrySet{struct_ident}{camel_cased_field_name}"),
        proc_macro2::Span::call_site(),
    );
    let try_field_name =
        syn::Ident::new(&format!("try_{field_name}"), proc_macro2::Span::call_site());
    let try_field_name_ref = syn::Ident::new(
        &format!("try_{field_name}_ref"),
        proc_macro2::Span::call_site(),
    );

    let try_set_trait_doc_comment =
        format!("Trait to try to set the `{field_name}` column on a table builder.");
    let try_field_name_ref_method_doc_comment =
        format!("Tries to set the `{field_name}` column on a table builder by reference.");
    let try_field_name_method_doc_comment =
        format!("Tries to set the `{field_name}` column on a table builder.");

    quote! {
        #[doc = #try_set_trait_doc_comment]
        pub trait #try_set_field_name: diesel_builders::TrySetColumn<#table_module::#field_name> + Sized {
            #[inline]
            #[doc = #try_field_name_ref_method_doc_comment]
            #[doc = ""]
            #[doc = " # Errors"]
            #[doc = ""]
            #[doc = "Returns an error if the column check constraints are not respected."]
            fn #try_field_name_ref(
                &mut self,
                value: impl Into<<#table_module::#field_name as diesel_builders::Typed>::ColumnType> + Clone
            ) -> Result<&mut Self, Self::Error> {
                use diesel_builders::TrySetColumnExt;
                self.try_set_column_ref::<#table_module::#field_name>(value)
            }
            #[inline]
            #[doc = #try_field_name_method_doc_comment]
            #[doc = ""]
            #[doc = " # Errors"]
            #[doc = ""]
            #[doc = "Returns an error if the value cannot be converted to the column type."]
            fn #try_field_name(
                self,
                value: impl Into<<#table_module::#field_name as diesel_builders::Typed>::ColumnType> + Clone
            ) -> Result<Self, Self::Error> {
                use diesel_builders::TrySetColumnExt;
                self.try_set_column::<#table_module::#field_name>(value)
            }
        }

        impl<T> #try_set_field_name for T where T: diesel_builders::TrySetColumn<#table_module::#field_name> {}
    }
}

/// Generate the Typed implementation for a field.
fn generate_typed_impl(
    field_name: &Ident,
    field_type: &syn::Type,
    table_module: &syn::Ident,
) -> TokenStream {
    // Determine the ValueType: if the column type is an Option<T>, ValueType = T,
    // otherwise ValueType = the field type itself.
    let value_type = extract_option_inner_type(field_type).unwrap_or(quote::quote! { #field_type });

    quote! {
        impl diesel_builders::Typed for #table_module::#field_name {
            type ValueType = #value_type;
            type ColumnType = #field_type;
        }
    }
}

/// Extract the inner type from `Option<T>`, returning `None` if not an Option.
fn extract_option_inner_type(field_type: &syn::Type) -> Option<TokenStream> {
    let syn::Type::Path(type_path) = field_type else {
        return None;
    };

    let segment = type_path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }

    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };

    let syn::GenericArgument::Type(inner) = args.args.first()? else {
        return None;
    };

    Some(quote::quote! { #inner })
}

#[allow(clippy::too_many_lines)]
/// Generate triangular relation traits for a field.
/// Only generates traits relevant to the field's mandatory/discretionary status.
fn generate_triangular_relation_traits(
    field_name: &Ident,
    table_module: &syn::Ident,
    struct_ident: &Ident,
    camel_cased_field_name: &str,
    is_mandatory: bool,
    is_discretionary: bool,
) -> TokenStream {
    let set_field_name_discretionary_model_trait = syn::Ident::new(
        &format!("Set{struct_ident}{camel_cased_field_name}DiscretionaryModel"),
        proc_macro2::Span::call_site(),
    );
    // Base method name: if column ends with `_id` strip it (e.g., `c_id` -> `c`).
    // If it's an `_id` column, use the base name for model/builder methods (e.g., `.c()`),
    // otherwise generate `{field_name}_model` and `{field_name}_builder`.
    let base_field_name = {
        let s = field_name.to_string();
        if let Some(stripped) = s.strip_suffix("_id") {
            stripped.to_string()
        } else {
            s
        }
    };
    let is_id_col = field_name.to_string().ends_with("_id");
    // For model methods, always use `{base}_model` (even for `_id` columns) to avoid
    // generating the same method name for both builder and model methods which would
    // cause ambiguous trait method resolution in Rust.
    let set_field_name_model_method = syn::Ident::new(
        &format!("{base_field_name}_model"),
        proc_macro2::Span::call_site(),
    );
    let set_field_name_model_method_ref = syn::Ident::new(
        &format!("{base_field_name}_model_ref"),
        proc_macro2::Span::call_site(),
    );
    let try_set_field_name_model_method = syn::Ident::new(
        &format!("try_{base_field_name}_model"),
        proc_macro2::Span::call_site(),
    );
    let try_set_field_name_model_method_ref = syn::Ident::new(
        &format!("try_{base_field_name}_model_ref"),
        proc_macro2::Span::call_site(),
    );
    let set_field_name_builder_method_name = if is_id_col {
        base_field_name.clone()
    } else {
        format!("{base_field_name}_builder")
    };
    let set_field_name_builder_method = syn::Ident::new(
        &set_field_name_builder_method_name,
        proc_macro2::Span::call_site(),
    );
    let set_field_name_builder_method_ref_name = if is_id_col {
        format!("{base_field_name}_ref")
    } else {
        format!("{base_field_name}_builder_ref")
    };
    let set_field_name_builder_method_ref = syn::Ident::new(
        &set_field_name_builder_method_ref_name,
        proc_macro2::Span::call_site(),
    );
    let try_set_field_name_builder_method_name = if is_id_col {
        format!("try_{base_field_name}")
    } else {
        format!("try_{base_field_name}_builder")
    };
    let try_set_field_name_builder_method = syn::Ident::new(
        &try_set_field_name_builder_method_name,
        proc_macro2::Span::call_site(),
    );
    let try_set_field_name_builder_method_ref_name = if is_id_col {
        format!("try_{base_field_name}_ref")
    } else {
        format!("try_{base_field_name}_builder_ref")
    };
    let try_set_field_name_builder_method_ref = syn::Ident::new(
        &try_set_field_name_builder_method_ref_name,
        proc_macro2::Span::call_site(),
    );
    let set_field_name_mandatory_builder_trait = syn::Ident::new(
        &format!("Set{struct_ident}{camel_cased_field_name}MandatoryBuilder"),
        proc_macro2::Span::call_site(),
    );
    let set_field_name_discretionary_builder_trait = syn::Ident::new(
        &format!("Set{struct_ident}{camel_cased_field_name}DiscretionaryBuilder"),
        proc_macro2::Span::call_site(),
    );
    let try_set_field_name_discretionary_model_trait = syn::Ident::new(
        &format!("TrySet{struct_ident}{camel_cased_field_name}DiscretionaryModel"),
        proc_macro2::Span::call_site(),
    );
    let try_set_field_name_mandatory_builder_trait = syn::Ident::new(
        &format!("TrySet{struct_ident}{camel_cased_field_name}MandatoryBuilder"),
        proc_macro2::Span::call_site(),
    );
    let try_set_field_name_discretionary_builder_trait = syn::Ident::new(
        &format!("TrySet{struct_ident}{camel_cased_field_name}DiscretionaryBuilder"),
        proc_macro2::Span::call_site(),
    );

    let set_discretionary_model_trait_doc_comment = format!(
        "Trait to set the `{field_name}` column model on a table builder relative to a discretionary triangular relation."
    );
    let set_discretionary_model_method_doc_comment = format!(
        "Sets the `{field_name}` column model on a table builder relative to a discretionary triangular relation."
    );
    let set_mandatory_builder_trait_doc_comment = format!(
        "Trait to set the `{field_name}` column builder on a table builder relative to a mandatory triangular relation."
    );
    let set_discretionary_builder_trait_doc_comment = format!(
        "Trait to set the `{field_name}` column builder on a table builder relative to a discretionary triangular relation."
    );
    let set_mandatory_builder_method_doc_comment = format!(
        "Sets the `{field_name}` column builder on a table builder relative to a mandatory triangular relation."
    );
    let set_discretionary_builder_method_doc_comment = format!(
        "Sets the `{field_name}` column builder on a table builder relative to a discretionary triangular relation."
    );
    let try_set_discretionary_model_trait_doc_comment = format!(
        "Trait to try to set the `{field_name}` column model on a table builder relative to a discretionary triangular relation."
    );
    let try_set_discretionary_model_method_doc_comment = format!(
        "Tries to set the `{field_name}` column model on a table builder relative to a discretionary triangular relation."
    );
    let try_set_mandatory_builder_trait_doc_comment = format!(
        "Trait to try to set the `{field_name}` column builder on a table builder relative to a mandatory triangular relation."
    );
    let try_set_discretionary_builder_trait_doc_comment = format!(
        "Trait to try to set the `{field_name}` column builder on a table builder relative to a discretionary triangular relation."
    );
    let try_set_mandatory_builder_method_doc_comment = format!(
        "Tries to set the `{field_name}` column builder on a table builder relative to a mandatory triangular relation."
    );
    let try_set_discretionary_builder_method_doc_comment = format!(
        "Tries to set the `{field_name}` column builder on a table builder relative to a discretionary triangular relation."
    );

    // Generate discretionary traits only if the field is marked as discretionary
    let discretionary_traits = if is_discretionary {
        quote! {
            #[doc = #set_discretionary_model_trait_doc_comment]
            pub trait #set_field_name_discretionary_model_trait: diesel_builders::SetDiscretionaryModel<#table_module::#field_name> + Sized
            {
                #[inline]
                #[doc = #set_discretionary_model_method_doc_comment]
                fn #set_field_name_model_method_ref(
                    &mut self,
                    value: &<<#table_module::#field_name as diesel_builders::ForeignPrimaryKey>::ReferencedTable as diesel_builders::TableExt>::Model
                ) -> &mut Self {
                    use diesel_builders::SetDiscretionaryModelExt;
                    self.set_discretionary_model_ref::<#table_module::#field_name>(value)
                }
                #[inline]
                #[must_use]
                #[doc = #set_discretionary_model_method_doc_comment]
                fn #set_field_name_model_method(
                    self,
                    value: &<<#table_module::#field_name as diesel_builders::ForeignPrimaryKey>::ReferencedTable as diesel_builders::TableExt>::Model
                ) -> Self {
                    use diesel_builders::SetDiscretionaryModelExt;
                    self.set_discretionary_model::<#table_module::#field_name>(value)
                }
            }

            impl<T> #set_field_name_discretionary_model_trait for T
                where
                    T: diesel_builders::SetDiscretionaryModel<#table_module::#field_name>
                {}

            #[doc = #set_discretionary_builder_trait_doc_comment]
            pub trait #set_field_name_discretionary_builder_trait: diesel_builders::SetDiscretionaryBuilder<#table_module::#field_name> + Sized
            {
                #[inline]
                #[doc = #set_discretionary_builder_method_doc_comment]
                fn #set_field_name_builder_method_ref(
                    &mut self,
                    value: diesel_builders::TableBuilder<<#table_module::#field_name as diesel_builders::ForeignPrimaryKey>::ReferencedTable>
                ) -> &mut Self {
                    use diesel_builders::SetDiscretionaryBuilderExt;
                    self.set_discretionary_builder_ref::<#table_module::#field_name>(value)
                }
                #[inline]
                #[must_use]
                #[doc = #set_discretionary_builder_method_doc_comment]
                fn #set_field_name_builder_method(
                    self,
                    value: diesel_builders::TableBuilder<<#table_module::#field_name as diesel_builders::ForeignPrimaryKey>::ReferencedTable>
                ) -> Self {
                    use diesel_builders::SetDiscretionaryBuilderExt;
                    self.set_discretionary_builder::<#table_module::#field_name>(value)
                }
            }

            impl<T> #set_field_name_discretionary_builder_trait for T
            where
                T: diesel_builders::SetDiscretionaryBuilder<#table_module::#field_name>
                {}

            #[doc = #try_set_discretionary_model_trait_doc_comment]
            pub trait #try_set_field_name_discretionary_model_trait: diesel_builders::TrySetDiscretionaryModel<#table_module::#field_name> + Sized
            {
                #[inline]
                #[doc = #try_set_discretionary_model_method_doc_comment]
                #[doc = ""]
                #[doc = " # Errors"]
                #[doc = ""]
                #[doc = "Returns an error if the column check constraints are not respected."]
                fn #try_set_field_name_model_method_ref(
                    &mut self,
                    value: &<<#table_module::#field_name as diesel_builders::ForeignPrimaryKey>::ReferencedTable as diesel_builders::TableExt>::Model
                ) -> Result<&mut Self, <Self::Table as diesel_builders::TableExt>::Error> {
                    use diesel_builders::TrySetDiscretionaryModelExt;
                    self.try_set_discretionary_model_ref::<#table_module::#field_name>(value)
                }
                #[inline]
                #[doc = #try_set_discretionary_model_method_doc_comment]
                #[doc = ""]
                #[doc = " # Errors"]
                #[doc = ""]
                #[doc = "Returns an error if the value cannot be converted to the column type."]
                fn #try_set_field_name_model_method(
                    self,
                    value: &<<#table_module::#field_name as diesel_builders::ForeignPrimaryKey>::ReferencedTable as diesel_builders::TableExt>::Model
                ) -> Result<Self, <Self::Table as diesel_builders::TableExt>::Error> {
                    use diesel_builders::TrySetDiscretionaryModelExt;
                    self.try_set_discretionary_model::<#table_module::#field_name>(value)
                }
            }

            impl<T> #try_set_field_name_discretionary_model_trait for T
            where
                T: diesel_builders::TrySetDiscretionaryModel<#table_module::#field_name>
                {}

            #[doc = #try_set_discretionary_builder_trait_doc_comment]
            pub trait #try_set_field_name_discretionary_builder_trait: diesel_builders::TrySetDiscretionaryBuilder<#table_module::#field_name> + Sized
            {
                #[inline]
                #[doc = #try_set_discretionary_builder_method_doc_comment]
                #[doc = ""]
                #[doc = " # Errors"]
                #[doc = ""]
                #[doc = "Returns an error if the column check constraints are not respected."]
                fn #try_set_field_name_builder_method_ref(
                    &mut self,
                    value: diesel_builders::TableBuilder<<#table_module::#field_name as diesel_builders::ForeignPrimaryKey>::ReferencedTable>
                ) -> Result<&mut Self, <Self::Table as diesel_builders::TableExt>::Error> {
                    use diesel_builders::TrySetDiscretionaryBuilderExt;
                    self.try_set_discretionary_builder_ref::<#table_module::#field_name>(value)
                }
                #[inline]
                #[doc = #try_set_discretionary_builder_method_doc_comment]
                #[doc = ""]
                #[doc = " # Errors"]
                #[doc = ""]
                #[doc = "Returns an error if the value cannot be converted to the column type."]
                fn #try_set_field_name_builder_method(
                    self,
                    value: diesel_builders::TableBuilder<<#table_module::#field_name as diesel_builders::ForeignPrimaryKey>::ReferencedTable>
                ) -> Result<Self, <Self::Table as diesel_builders::TableExt>::Error> {
                    use diesel_builders::TrySetDiscretionaryBuilderExt;
                    self.try_set_discretionary_builder::<#table_module::#field_name>(value)
                }
            }

            impl<T> #try_set_field_name_discretionary_builder_trait for T
            where
                T: diesel_builders::TrySetDiscretionaryBuilder<#table_module::#field_name>
                {}
        }
    } else {
        quote! {}
    };

    // Generate mandatory traits only if the field is marked as mandatory
    let mandatory_traits = if is_mandatory {
        quote! {
            #[doc = #set_mandatory_builder_trait_doc_comment]
            pub trait #set_field_name_mandatory_builder_trait: diesel_builders::SetMandatoryBuilder<#table_module::#field_name> + Sized
            {
                #[inline]
                #[doc = #set_mandatory_builder_method_doc_comment]
                fn #set_field_name_builder_method_ref(
                    &mut self,
                    value: diesel_builders::TableBuilder<<#table_module::#field_name as diesel_builders::ForeignPrimaryKey>::ReferencedTable>
                ) -> &mut Self {
                    use diesel_builders::SetMandatoryBuilderExt;
                    self.set_mandatory_builder_ref::<#table_module::#field_name>(value)
                }
                #[inline]
                #[must_use]
                #[doc = #set_mandatory_builder_method_doc_comment]
                fn #set_field_name_builder_method(
                    self,
                    value: diesel_builders::TableBuilder<<#table_module::#field_name as diesel_builders::ForeignPrimaryKey>::ReferencedTable>
                ) -> Self {
                    use diesel_builders::SetMandatoryBuilderExt;
                    self.set_mandatory_builder::<#table_module::#field_name>(value)
                }
            }

            impl<T> #set_field_name_mandatory_builder_trait for T
            where
                T: diesel_builders::SetMandatoryBuilder<#table_module::#field_name>
                {}

            #[doc = #try_set_mandatory_builder_trait_doc_comment]
            pub trait #try_set_field_name_mandatory_builder_trait: diesel_builders::TrySetMandatoryBuilder<#table_module::#field_name> + Sized
            {
                #[inline]
                #[doc = #try_set_mandatory_builder_method_doc_comment]
                #[doc = ""]
                #[doc = " # Errors"]
                #[doc = ""]
                #[doc = "Returns an error if the column check constraints are not respected."]
                fn #try_set_field_name_builder_method_ref(
                    &mut self,
                    value: diesel_builders::TableBuilder<<#table_module::#field_name as diesel_builders::ForeignPrimaryKey>::ReferencedTable>
                ) -> Result<&mut Self, <Self::Table as diesel_builders::TableExt>::Error> {
                    use diesel_builders::TrySetMandatoryBuilderExt;
                    self.try_set_mandatory_builder_ref::<#table_module::#field_name>(value)
                }
                #[inline]
                #[doc = #try_set_mandatory_builder_method_doc_comment]
                #[doc = ""]
                #[doc = " # Errors"]
                #[doc = ""]
                #[doc = "Returns an error if the value cannot be converted to the column type."]
                fn #try_set_field_name_builder_method(
                    self,
                    value: diesel_builders::TableBuilder<<#table_module::#field_name as diesel_builders::ForeignPrimaryKey>::ReferencedTable>
                ) -> Result<Self, <Self::Table as diesel_builders::TableExt>::Error> {
                    use diesel_builders::TrySetMandatoryBuilderExt;
                    self.try_set_mandatory_builder::<#table_module::#field_name>(value)
                }
            }

            impl<T> #try_set_field_name_mandatory_builder_trait for T
            where
                T: diesel_builders::TrySetMandatoryBuilder<#table_module::#field_name>
                {}
        }
    } else {
        quote! {}
    };

    quote! {
        #discretionary_traits
        #mandatory_traits
    }
}
