//! `TypedColumn` trait implementations and associated setter/getter traits.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, Ident, Token, punctuated::Punctuated};

use crate::utils::snake_to_camel_case;

/// Builds an identifier from `name` using call-site hygiene.
fn ident(name: &str) -> Ident {
    Ident::new(name, proc_macro2::Span::call_site())
}

/// Emits a marker trait carrying `methods` together with its blanket
/// implementation for every `T` that satisfies `bound`.
///
/// The generated setter and getter surface is a family of `pub trait … : Bound`
/// declarations each paired with an `impl<T> … for T where T: Bound {}`, so
/// this centralises that scaffold and leaves every caller to supply only the
/// per-trait methods.
fn trait_with_blanket_impl(
    name: &Ident,
    bound: &TokenStream,
    doc: &str,
    methods: &TokenStream,
) -> TokenStream {
    quote! {
        #[doc = #doc]
        pub trait #name: #bound {
            #methods
        }
        impl<T> #name for T where T: #bound {}
    }
}

/// Generate `TypedColumn` implementations and associated setter/getter traits
/// for all fields.
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

    let field_name_str = field_name.to_string();
    let clean_field_name = field_name_str.trim_start_matches("r#");

    let method_name_str = clean_field_name.trim_start_matches('_').to_string();
    let method_name_ident = syn::parse_str::<syn::Ident>(&method_name_str)
        .unwrap_or_else(|_| syn::Ident::new_raw(&method_name_str, proc_macro2::Span::call_site()));

    let camel_cased_field_name = snake_to_camel_case(&method_name_str);

    // Generate getter trait only for non-id fields
    let maybe_getter_impl = (field_name != "id").then(|| {
        generate_getter_trait(
            field_name,
            &method_name_ident,
            table_module,
            struct_ident,
            &camel_cased_field_name,
        )
    });

    // Determine triangular relation type
    let is_mandatory = is_field_mandatory(field);
    let is_discretionary = is_field_discretionary(field);

    // Generate triangular relation traits only for single primary key tables
    // and if field is marked
    let maybe_triangular_impls =
        if primary_key_columns.len() == 1 && (is_mandatory || is_discretionary) {
            Some(generate_triangular_relation_traits(
                field_name,
                &method_name_str,
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
        &method_name_str,
        &method_name_ident,
        table_module,
        struct_ident,
        &camel_cased_field_name,
    );
    let try_set_trait = generate_try_set_trait(
        field_name,
        &method_name_str,
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
    method_name: &Ident,
    table_module: &syn::Ident,
    struct_ident: &Ident,
    camel_cased_field_name: &str,
) -> TokenStream {
    let get_field_name = ident(&format!("Get{struct_ident}{camel_cased_field_name}"));

    let get_trait_doc_comment =
        format!("Trait to get the `{field_name}` column from a `{table_module}` table model.");
    let get_field_name_method_doc_comment =
        format!("Gets the value of the `{field_name}` column from a `{table_module}` table model.");

    let bound = quote!(::diesel_builders::GetColumn<#table_module::#field_name>);
    let methods = quote! {
        #[inline]
        #[doc = #get_field_name_method_doc_comment]
        fn #method_name(&self) -> &<#table_module::#field_name as ::diesel_builders::ColumnTyped>::ColumnType {
            self.get_column_ref()
        }
    };

    trait_with_blanket_impl(&get_field_name, &bound, &get_trait_doc_comment, &methods)
}

/// Generate the `SetColumn` trait for a field.
fn generate_set_trait(
    field_name: &Ident,
    clean_field_name: &str,
    method_name_ident: &Ident,
    table_module: &syn::Ident,
    struct_ident: &Ident,
    camel_cased_field_name: &str,
) -> TokenStream {
    let set_field_name = ident(&format!("Set{struct_ident}{camel_cased_field_name}"));
    let field_name_ref = ident(&format!("{clean_field_name}_ref"));
    let method_name = method_name_ident;

    let set_trait_doc_comment =
        format!("Trait to set the `{field_name}` column on a [`{table_module}`] table builder.");
    let field_name_ref_method_doc_comment = format!(
        "Sets the `{field_name}` column on a [`{table_module}`] table builder by reference."
    );
    let field_name_method_doc_comment =
        format!("Sets the `{field_name}` column on a [`{table_module}`] table builder.");

    let bound = quote!(diesel_builders::SetColumn<#table_module::#field_name> + Sized);
    let methods = quote! {
        #[inline]
        #[doc = #field_name_ref_method_doc_comment]
        fn #field_name_ref(
            &mut self,
            value: impl Into<<#table_module::#field_name as ::diesel_builders::ColumnTyped>::ColumnType>
        ) -> &mut Self {
            use diesel_builders::SetColumnExt;
            self.set_column_ref::<#table_module::#field_name>(value)
        }
        #[inline]
        #[must_use]
        #[doc = #field_name_method_doc_comment]
        fn #method_name(
            self,
            value: impl Into<<#table_module::#field_name as ::diesel_builders::ColumnTyped>::ColumnType>
        ) -> Self {
            use diesel_builders::SetColumnExt;
            self.set_column::<#table_module::#field_name>(value)
        }
    };

    trait_with_blanket_impl(&set_field_name, &bound, &set_trait_doc_comment, &methods)
}

/// Generate the `TrySetColumn` trait for a field.
fn generate_try_set_trait(
    field_name: &Ident,
    clean_field_name: &str,
    table_module: &syn::Ident,
    struct_ident: &Ident,
    camel_cased_field_name: &str,
) -> TokenStream {
    let try_set_field_name = ident(&format!("TrySet{struct_ident}{camel_cased_field_name}"));
    let try_field_name = ident(&format!("try_{clean_field_name}"));
    let try_field_name_ref = ident(&format!("try_{clean_field_name}_ref"));

    let try_set_trait_doc_comment =
        format!("Trait to try to set the `{field_name}` column on a table builder.");
    let try_field_name_ref_method_doc_comment =
        format!("Tries to set the `{field_name}` column on a table builder by reference.");
    let try_field_name_method_doc_comment =
        format!("Tries to set the `{field_name}` column on a table builder.");

    let bound = quote!(diesel_builders::TrySetColumn<#table_module::#field_name> + Sized);
    let methods = quote! {
        #[inline]
        #[doc = #try_field_name_ref_method_doc_comment]
        #[doc = ""]
        #[doc = " # Errors"]
        #[doc = ""]
        #[doc = "Returns an error if the column check constraints are not respected."]
        fn #try_field_name_ref(
            &mut self,
            value: impl Into<<#table_module::#field_name as ::diesel_builders::ColumnTyped>::ColumnType> + Clone
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
            value: impl Into<<#table_module::#field_name as ::diesel_builders::ColumnTyped>::ColumnType> + Clone
        ) -> Result<Self, Self::Error> {
            use diesel_builders::TrySetColumnExt;
            self.try_set_column::<#table_module::#field_name>(value)
        }
    };

    trait_with_blanket_impl(&try_set_field_name, &bound, &try_set_trait_doc_comment, &methods)
}

/// Generate the Typed implementation for a field.
fn generate_typed_impl(
    field_name: &Ident,
    field_type: &syn::Type,
    table_module: &syn::Ident,
) -> TokenStream {
    // Determine the ValueType: if the column type is an Option<T>, ValueType =
    // T, otherwise ValueType = the field type itself.
    let value_type = extract_option_inner_type(field_type).unwrap_or(quote::quote! { #field_type });

    quote! {
        impl ::diesel_builders::ValueTyped for #table_module::#field_name {
            type ValueType = #value_type;
        }
        impl ::diesel_builders::ColumnTyped for #table_module::#field_name {
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
/// Only generates traits relevant to the field's mandatory/discretionary
/// status.
fn generate_triangular_relation_traits(
    field_name: &Ident,
    clean_field_name: &str,
    table_module: &syn::Ident,
    struct_ident: &Ident,
    camel_cased_field_name: &str,
    is_mandatory: bool,
    is_discretionary: bool,
) -> TokenStream {
    let set_field_name_discretionary_model_trait =
        ident(&format!("Set{struct_ident}{camel_cased_field_name}DiscretionaryModel"));
    // Base method name: if column ends with `_id` strip it (e.g., `c_id` ->
    // `c`). If it's an `_id` column, use the base name for model/builder
    // methods (e.g., `.c()`), otherwise generate `{field_name}_model` and
    // `{field_name}_builder`.
    let base_field_name = {
        let s = field_name.to_string();
        if let Some(stripped) = s.strip_suffix("_id") { stripped.to_string() } else { s }
    };
    let clean_base_field_name = {
        let s = clean_field_name;
        if let Some(stripped) = s.strip_suffix("_id") { stripped } else { s }
    };
    let is_id_col = field_name.to_string().ends_with("_id");
    // For model methods, always use `{base}_model` (even for `_id` columns) to
    // avoid generating the same method name for both builder and model methods
    // which would cause ambiguous trait method resolution in Rust.
    let set_field_name_model_method = ident(&format!("{clean_base_field_name}_model"));
    let set_field_name_model_method_ref = ident(&format!("{clean_base_field_name}_model_ref"));
    let try_set_field_name_model_method = ident(&format!("try_{clean_base_field_name}_model"));
    let try_set_field_name_model_method_ref =
        ident(&format!("try_{clean_base_field_name}_model_ref"));
    let set_field_name_builder_method_name =
        if is_id_col { base_field_name } else { format!("{clean_base_field_name}_builder") };
    let set_field_name_builder_method = ident(&set_field_name_builder_method_name);
    let set_field_name_builder_method_ref_name = if is_id_col {
        format!("{clean_base_field_name}_ref")
    } else {
        format!("{clean_base_field_name}_builder_ref")
    };
    let set_field_name_builder_method_ref = ident(&set_field_name_builder_method_ref_name);
    let try_set_field_name_builder_method_name = if is_id_col {
        format!("try_{clean_base_field_name}")
    } else {
        format!("try_{clean_base_field_name}_builder")
    };
    let try_set_field_name_builder_method = ident(&try_set_field_name_builder_method_name);
    let try_set_field_name_builder_method_ref_name = if is_id_col {
        format!("try_{clean_base_field_name}_ref")
    } else {
        format!("try_{clean_base_field_name}_builder_ref")
    };
    let try_set_field_name_builder_method_ref = ident(&try_set_field_name_builder_method_ref_name);
    let set_field_name_mandatory_builder_trait =
        ident(&format!("Set{struct_ident}{camel_cased_field_name}MandatoryBuilder"));
    let set_field_name_discretionary_builder_trait =
        ident(&format!("Set{struct_ident}{camel_cased_field_name}DiscretionaryBuilder"));
    let try_set_field_name_discretionary_model_trait =
        ident(&format!("TrySet{struct_ident}{camel_cased_field_name}DiscretionaryModel"));
    let try_set_field_name_mandatory_builder_trait =
        ident(&format!("TrySet{struct_ident}{camel_cased_field_name}MandatoryBuilder"));
    let try_set_field_name_discretionary_builder_trait =
        ident(&format!("TrySet{struct_ident}{camel_cased_field_name}DiscretionaryBuilder"));

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

    // Generate discretionary traits only if the field is marked as
    // discretionary
    let discretionary_traits = if is_discretionary {
        let discretionary_model = trait_with_blanket_impl(
            &set_field_name_discretionary_model_trait,
            &quote!(diesel_builders::SetDiscretionaryModel<#table_module::#field_name> + Sized),
            &set_discretionary_model_trait_doc_comment,
            &quote! {
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
            },
        );

        let discretionary_builder = trait_with_blanket_impl(
            &set_field_name_discretionary_builder_trait,
            &quote!(diesel_builders::SetDiscretionaryBuilder<#table_module::#field_name> + Sized),
            &set_discretionary_builder_trait_doc_comment,
            &quote! {
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
            },
        );

        let try_discretionary_model = trait_with_blanket_impl(
            &try_set_field_name_discretionary_model_trait,
            &quote!(diesel_builders::TrySetDiscretionaryModel<#table_module::#field_name> + Sized),
            &try_set_discretionary_model_trait_doc_comment,
            &quote! {
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
            },
        );

        let try_discretionary_builder = trait_with_blanket_impl(
            &try_set_field_name_discretionary_builder_trait,
            &quote!(diesel_builders::TrySetDiscretionaryBuilder<#table_module::#field_name> + Sized),
            &try_set_discretionary_builder_trait_doc_comment,
            &quote! {
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
            },
        );

        quote! {
            #discretionary_model
            #discretionary_builder
            #try_discretionary_model
            #try_discretionary_builder
        }
    } else {
        quote! {}
    };

    // Generate mandatory traits only if the field is marked as mandatory
    let mandatory_traits = if is_mandatory {
        let mandatory_builder = trait_with_blanket_impl(
            &set_field_name_mandatory_builder_trait,
            &quote!(diesel_builders::SetMandatoryBuilder<#table_module::#field_name> + Sized),
            &set_mandatory_builder_trait_doc_comment,
            &quote! {
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
            },
        );

        let try_mandatory_builder = trait_with_blanket_impl(
            &try_set_field_name_mandatory_builder_trait,
            &quote!(diesel_builders::TrySetMandatoryBuilder<#table_module::#field_name> + Sized),
            &try_set_mandatory_builder_trait_doc_comment,
            &quote! {
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
            },
        );

        quote! {
            #mandatory_builder
            #try_mandatory_builder
        }
    } else {
        quote! {}
    };

    quote! {
        #discretionary_traits
        #mandatory_traits
    }
}
