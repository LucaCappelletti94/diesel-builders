//! `TypedColumn` trait implementations and associated setter/getter traits.

use crate::utils::snake_to_camel_case;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, Field, Ident, Token};

/// Generate `TypedColumn` implementations and associated setter/getter traits for all fields.
pub fn generate_typed_column_impls(
    fields: &Punctuated<Field, Token![,]>,
    table_name: &Ident,
    struct_ident: &Ident,
    primary_key_columns: &[Ident],
) -> TokenStream {
    fields.iter().filter_map(|field| {
        let field_name = field.ident.as_ref()?;
        let field_type = &field.ty;
        
        Some(generate_field_traits(
            field_name,
            field_type,
            table_name,
            struct_ident,
            primary_key_columns,
        ))
    }).collect()
}

/// Generate all trait implementations for a single field.
fn generate_field_traits(
    field_name: &Ident,
    field_type: &syn::Type,
    table_name: &Ident,
    struct_ident: &Ident,
    primary_key_columns: &[Ident],
) -> TokenStream {
    let camel_cased_field_name = snake_to_camel_case(&field_name.to_string());

    // Generate getter trait only for non-id fields
    let maybe_getter_impl = (field_name != "id").then(|| {
        generate_getter_trait(field_name, table_name, struct_ident, &camel_cased_field_name)
    });

    // Generate triangular relation traits only for single primary key tables
    let maybe_triangular_impls = (primary_key_columns.len() == 1).then(|| {
        generate_triangular_relation_traits(
            field_name,
            table_name,
            struct_ident,
            &camel_cased_field_name,
        )
    });

    let set_trait = generate_set_trait(field_name, table_name, struct_ident, &camel_cased_field_name);
    let try_set_trait = generate_try_set_trait(field_name, table_name, struct_ident, &camel_cased_field_name);
    let typed_impl = generate_typed_impl(field_name, field_type, table_name);

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
    table_name: &Ident,
    struct_ident: &Ident,
    camel_cased_field_name: &str,
) -> TokenStream {
    let get_field_name = syn::Ident::new(
        &format!("Get{struct_ident}{camel_cased_field_name}"),
        proc_macro2::Span::call_site(),
    );

    let get_trait_doc_comment = format!(
        "Trait to get the `{field_name}` column from a `{table_name}` table model."
    );
    let get_field_name_method_doc_comment = format!(
        "Gets the value of the `{field_name}` column from a `{table_name}` table model."
    );

    quote! {
        #[doc = #get_trait_doc_comment]
        pub trait #get_field_name: diesel_builders::GetColumn<#table_name::#field_name> {
            #[inline]
            #[doc = #get_field_name_method_doc_comment]
            fn #field_name(&self) -> &<#table_name::#field_name as diesel_builders::Typed>::Type {
                self.get_column_ref()
            }
        }
        impl<T> #get_field_name for T where T: diesel_builders::GetColumn<#table_name::#field_name> {}
    }
}

/// Generate the `SetColumn` trait for a field.
fn generate_set_trait(
    field_name: &Ident,
    table_name: &Ident,
    struct_ident: &Ident,
    camel_cased_field_name: &str,
) -> TokenStream {
    let set_field_name = syn::Ident::new(
        &format!("Set{struct_ident}{camel_cased_field_name}"),
        proc_macro2::Span::call_site(),
    );
    let field_name_ref = syn::Ident::new(
        &format!("{field_name}_ref"),
        proc_macro2::Span::call_site(),
    );

    let set_trait_doc_comment = format!(
        "Trait to set the `{field_name}` column on a `{table_name}` table builder."
    );
    let field_name_ref_method_doc_comment = format!(
        "Sets the `{field_name}` column on a `{table_name}` table builder by reference."
    );
    let field_name_method_doc_comment = format!(
        "Sets the `{field_name}` column on a `{table_name}` table builder."
    );

    quote! {
        #[doc = #set_trait_doc_comment]
        pub trait #set_field_name: diesel_builders::SetColumn<#table_name::#field_name> + Sized {
            #[inline]
            #[doc = #field_name_ref_method_doc_comment]
            fn #field_name_ref(
                &mut self,
                value: impl Into<<#table_name::#field_name as diesel_builders::Typed>::Type>
            ) -> &mut Self {
                use diesel_builders::SetColumnExt;
                self.set_column_ref::<#table_name::#field_name>(value)
            }
            #[inline]
            #[must_use]
            #[doc = #field_name_method_doc_comment]
            fn #field_name(
                self,
                value: impl Into<<#table_name::#field_name as diesel_builders::Typed>::Type>
            ) -> Self {
                use diesel_builders::SetColumnExt;
                self.set_column::<#table_name::#field_name>(value)
            }
        }

        impl<T> #set_field_name for T where T: diesel_builders::SetColumn<#table_name::#field_name> {}
    }
}

/// Generate the `TrySetColumn` trait for a field.
fn generate_try_set_trait(
    field_name: &Ident,
    table_name: &Ident,
    struct_ident: &Ident,
    camel_cased_field_name: &str,
) -> TokenStream {
    let try_set_field_name = syn::Ident::new(
        &format!("TrySet{struct_ident}{camel_cased_field_name}"),
        proc_macro2::Span::call_site(),
    );
    let try_field_name = syn::Ident::new(
        &format!("try_{field_name}"),
        proc_macro2::Span::call_site(),
    );
    let try_field_name_ref = syn::Ident::new(
        &format!("try_{field_name}_ref"),
        proc_macro2::Span::call_site(),
    );

    let try_set_trait_doc_comment = format!(
        "Trait to try to set the `{field_name}` column on a `{table_name}` table builder."
    );
    let try_field_name_ref_method_doc_comment = format!(
        "Tries to set the `{field_name}` column on a `{table_name}` table builder by reference."
    );
    let try_field_name_method_doc_comment = format!(
        "Tries to set the `{field_name}` column on a `{table_name}` table builder."
    );

    quote! {
        #[doc = #try_set_trait_doc_comment]
        pub trait #try_set_field_name: diesel_builders::TrySetColumn<#table_name::#field_name> + Sized {
            #[inline]
            #[doc = #try_field_name_ref_method_doc_comment]
            #[doc = ""]
            #[doc = " # Errors"]
            #[doc = ""]
            #[doc = "Returns an error if the column check constraints are not respected."]
            fn #try_field_name_ref(
                &mut self,
                value: impl Into<<#table_name::#field_name as diesel_builders::Typed>::Type>
            ) -> Result<&mut Self, Self::Error> {
                use diesel_builders::TrySetColumnExt;
                self.try_set_column_ref::<#table_name::#field_name>(value)
            }
            #[inline]
            #[doc = #try_field_name_method_doc_comment]
            #[doc = ""]
            #[doc = " # Errors"]
            #[doc = ""]
            #[doc = "Returns an error if the value cannot be converted to the column type."]
            fn #try_field_name(
                self,
                value: impl Into<<#table_name::#field_name as diesel_builders::Typed>::Type>
            ) -> Result<Self, Self::Error> {
                use diesel_builders::TrySetColumnExt;
                self.try_set_column::<#table_name::#field_name>(value)
            }
        }

        impl<T> #try_set_field_name for T where T: diesel_builders::TrySetColumn<#table_name::#field_name> {}
    }
}

/// Generate the Typed implementation for a field.
fn generate_typed_impl(
    field_name: &Ident,
    field_type: &syn::Type,
    table_name: &Ident,
) -> TokenStream {
    quote! {
        impl diesel_builders::Typed for #table_name::#field_name {
            type Type = #field_type;
        }
    }
}

#[allow(clippy::too_many_lines)]
/// Generate triangular relation traits for a field.
fn generate_triangular_relation_traits(
    field_name: &Ident,
    table_name: &Ident,
    struct_ident: &Ident,
    camel_cased_field_name: &str,
) -> TokenStream {
    let set_field_name_discretionary_model_trait = syn::Ident::new(
        &format!("Set{struct_ident}{camel_cased_field_name}DiscretionaryModel"),
        proc_macro2::Span::call_site(),
    );
    let set_field_name_model_method = syn::Ident::new(
        &format!("{field_name}_model"),
        proc_macro2::Span::call_site(),
    );
    let set_field_name_model_method_ref = syn::Ident::new(
        &format!("{field_name}_model_ref"),
        proc_macro2::Span::call_site(),
    );
    let try_set_field_name_model_method = syn::Ident::new(
        &format!("try_{field_name}_model"),
        proc_macro2::Span::call_site(),
    );
    let try_set_field_name_model_method_ref = syn::Ident::new(
        &format!("try_{field_name}_model_ref"),
        proc_macro2::Span::call_site(),
    );
    let set_field_name_builder_method = syn::Ident::new(
        &format!("{field_name}_builder"),
        proc_macro2::Span::call_site(),
    );
    let set_field_name_builder_method_ref = syn::Ident::new(
        &format!("{field_name}_builder_ref"),
        proc_macro2::Span::call_site(),
    );
    let try_set_field_name_builder_method = syn::Ident::new(
        &format!("try_{field_name}_builder"),
        proc_macro2::Span::call_site(),
    );
    let try_set_field_name_builder_method_ref = syn::Ident::new(
        &format!("try_{field_name}_builder_ref"),
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
        "Trait to set the `{field_name}` column model on a `{table_name}` table builder relative to a discretionary triangular relation."
    );
    let set_discretionary_model_method_doc_comment = format!(
        "Sets the `{field_name}` column model on a `{table_name}` table builder relative to a discretionary triangular relation."
    );
    let set_mandatory_builder_trait_doc_comment = format!(
        "Trait to set the `{field_name}` column builder on a `{table_name}` table builder relative to a mandatory triangular relation."
    );
    let set_discretionary_builder_trait_doc_comment = format!(
        "Trait to set the `{field_name}` column builder on a `{table_name}` table builder relative to a discretionary triangular relation."
    );
    let set_mandatory_builder_method_doc_comment = format!(
        "Sets the `{field_name}` column builder on a `{table_name}` table builder relative to a mandatory triangular relation."
    );
    let set_discretionary_builder_method_doc_comment = format!(
        "Sets the `{field_name}` column builder on a `{table_name}` table builder relative to a discretionary triangular relation."
    );
    let try_set_discretionary_model_trait_doc_comment = format!(
        "Trait to try to set the `{field_name}` column model on a `{table_name}` table builder relative to a discretionary triangular relation."
    );
    let try_set_discretionary_model_method_doc_comment = format!(
        "Tries to set the `{field_name}` column model on a `{table_name}` table builder relative to a discretionary triangular relation."
    );
    let try_set_mandatory_builder_trait_doc_comment = format!(
        "Trait to try to set the `{field_name}` column builder on a `{table_name}` table builder relative to a mandatory triangular relation."
    );
    let try_set_discretionary_builder_trait_doc_comment = format!(
        "Trait to try to set the `{field_name}` column builder on a `{table_name}` table builder relative to a discretionary triangular relation."
    );
    let try_set_mandatory_builder_method_doc_comment = format!(
        "Tries to set the `{field_name}` column builder on a `{table_name}` table builder relative to a mandatory triangular relation."
    );
    let try_set_discretionary_builder_method_doc_comment = format!(
        "Tries to set the `{field_name}` column builder on a `{table_name}` table builder relative to a discretionary triangular relation."
    );

    quote! {
        #[doc = #set_discretionary_model_trait_doc_comment]
        pub trait #set_field_name_discretionary_model_trait: diesel_builders::SetDiscretionaryModel<#table_name::#field_name> + Sized
            where
                for<'a> #table_name::#field_name: diesel_builders::DiscretionarySameAsIndex
        {
            #[inline]
            #[doc = #set_discretionary_model_method_doc_comment]
            fn #set_field_name_model_method_ref(
                &mut self,
                value: &<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable as diesel_builders::TableAddition>::Model
            ) -> &mut Self {
                use diesel_builders::SetDiscretionaryModelExt;
                self.set_discretionary_model_ref::<#table_name::#field_name>(value)
            }
            #[inline]
            #[must_use]
            #[doc = #set_discretionary_model_method_doc_comment]
            fn #set_field_name_model_method(
                self,
                value: &<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable as diesel_builders::TableAddition>::Model
            ) -> Self {
                use diesel_builders::SetDiscretionaryModelExt;
                self.set_discretionary_model::<#table_name::#field_name>(value)
            }
        }

        impl<T> #set_field_name_discretionary_model_trait for T
            where
                T: diesel_builders::SetDiscretionaryModel<#table_name::#field_name>,
                for<'a> #table_name::#field_name: diesel_builders::DiscretionarySameAsIndex
            {}

        #[doc = #set_mandatory_builder_trait_doc_comment]
        pub trait #set_field_name_mandatory_builder_trait: diesel_builders::SetMandatoryBuilder<#table_name::#field_name> + Sized
            where
                for<'a> #table_name::#field_name: diesel_builders::MandatorySameAsIndex<ReferencedTable: BuildableTable>,
        {
            #[inline]
            #[doc = #set_mandatory_builder_method_doc_comment]
            fn #set_field_name_builder_method_ref(
                &mut self,
                value: diesel_builders::TableBuilder<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable>
            ) -> &mut Self {
                use diesel_builders::SetMandatoryBuilderExt;
                self.set_mandatory_builder_ref::<#table_name::#field_name>(value)
            }
            #[inline]
            #[must_use]
            #[doc = #set_mandatory_builder_method_doc_comment]
            fn #set_field_name_builder_method(
                self,
                value: diesel_builders::TableBuilder<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable>
            ) -> Self {
                use diesel_builders::SetMandatoryBuilderExt;
                self.set_mandatory_builder::<#table_name::#field_name>(value)
            }
        }

        impl<T> #set_field_name_mandatory_builder_trait for T
        where
            T: diesel_builders::SetMandatoryBuilder<#table_name::#field_name>,
            for<'a> #table_name::#field_name: diesel_builders::MandatorySameAsIndex<ReferencedTable: BuildableTable>,
            {}

        #[doc = #set_discretionary_builder_trait_doc_comment]
        pub trait #set_field_name_discretionary_builder_trait: diesel_builders::SetDiscretionaryBuilder<#table_name::#field_name> + Sized
            where
                for<'a> #table_name::#field_name: diesel_builders::DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
        {
            #[inline]
            #[doc = #set_discretionary_builder_method_doc_comment]
            fn #set_field_name_builder_method_ref(
                &mut self,
                value: diesel_builders::TableBuilder<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable>
            ) -> &mut Self {
                use diesel_builders::SetDiscretionaryBuilderExt;
                self.set_discretionary_builder_ref::<#table_name::#field_name>(value)
            }
            #[inline]
            #[must_use]
            #[doc = #set_discretionary_builder_method_doc_comment]
            fn #set_field_name_builder_method(
                self,
                value: diesel_builders::TableBuilder<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable>
            ) -> Self {
                use diesel_builders::SetDiscretionaryBuilderExt;
                self.set_discretionary_builder::<#table_name::#field_name>(value)
            }
        }

        impl<T> #set_field_name_discretionary_builder_trait for T
        where
            T: diesel_builders::SetDiscretionaryBuilder<#table_name::#field_name>,
            for<'a> #table_name::#field_name: diesel_builders::DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
            {}

        #[doc = #try_set_discretionary_model_trait_doc_comment]
        pub trait #try_set_field_name_discretionary_model_trait: diesel_builders::TrySetDiscretionaryModel<#table_name::#field_name> + Sized
        where
            for<'a> #table_name::#field_name: diesel_builders::DiscretionarySameAsIndex
        {
            #[inline]
            #[doc = #try_set_discretionary_model_method_doc_comment]
            #[doc = ""]
            #[doc = " # Errors"]
            #[doc = ""]
            #[doc = "Returns an error if the column check constraints are not respected."]
            fn #try_set_field_name_model_method_ref(
                &mut self,
                value: &<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable as diesel_builders::TableAddition>::Model
            ) -> Result<&mut Self, <<<Self as diesel::associations::HasTable>::Table as diesel_builders::TableAddition>::InsertableModel as diesel_builders::InsertableTableModel>::Error> {
                use diesel_builders::TrySetDiscretionaryModelExt;
                self.try_set_discretionary_model_ref::<#table_name::#field_name>(value)
            }
            #[inline]
            #[doc = #try_set_discretionary_model_method_doc_comment]
            #[doc = ""]
            #[doc = " # Errors"]
            #[doc = ""]
            #[doc = "Returns an error if the value cannot be converted to the column type."]
            fn #try_set_field_name_model_method(
                self,
                value: &<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable as diesel_builders::TableAddition>::Model
            ) -> Result<Self, <<<Self as diesel::associations::HasTable>::Table as diesel_builders::TableAddition>::InsertableModel as diesel_builders::InsertableTableModel>::Error> {
                use diesel_builders::TrySetDiscretionaryModelExt;
                self.try_set_discretionary_model::<#table_name::#field_name>(value)
            }
        }

        impl<T> #try_set_field_name_discretionary_model_trait for T
        where
            T: diesel_builders::TrySetDiscretionaryModel<#table_name::#field_name>,
            for<'a> #table_name::#field_name: diesel_builders::DiscretionarySameAsIndex
            {}

        #[doc = #try_set_mandatory_builder_trait_doc_comment]
        pub trait #try_set_field_name_mandatory_builder_trait: diesel_builders::TrySetMandatoryBuilder<#table_name::#field_name> + Sized
        where
            for<'a> #table_name::#field_name: diesel_builders::MandatorySameAsIndex<ReferencedTable: BuildableTable>,
        {
            #[inline]
            #[doc = #try_set_mandatory_builder_method_doc_comment]
            #[doc = ""]
            #[doc = " # Errors"]
            #[doc = ""]
            #[doc = "Returns an error if the column check constraints are not respected."]
            fn #try_set_field_name_builder_method_ref(
                &mut self,
                value: diesel_builders::TableBuilder<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable>
            ) -> Result<&mut Self, <<<Self as diesel::associations::HasTable>::Table as diesel_builders::TableAddition>::InsertableModel as diesel_builders::InsertableTableModel>::Error> {
                use diesel_builders::TrySetMandatoryBuilderExt;
                self.try_set_mandatory_builder_ref::<#table_name::#field_name>(value)
            }
            #[inline]
            #[doc = #try_set_mandatory_builder_method_doc_comment]
            #[doc = ""]
            #[doc = " # Errors"]
            #[doc = ""]
            #[doc = "Returns an error if the value cannot be converted to the column type."]
            fn #try_set_field_name_builder_method(
                self,
                value: diesel_builders::TableBuilder<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable>
            ) -> Result<Self, <<<Self as diesel::associations::HasTable>::Table as diesel_builders::TableAddition>::InsertableModel as diesel_builders::InsertableTableModel>::Error> {
                use diesel_builders::TrySetMandatoryBuilderExt;
                self.try_set_mandatory_builder::<#table_name::#field_name>(value)
            }
        }

        impl<T> #try_set_field_name_mandatory_builder_trait for T
        where
            T: diesel_builders::TrySetMandatoryBuilder<#table_name::#field_name>,
            for<'a> #table_name::#field_name: diesel_builders::MandatorySameAsIndex<ReferencedTable: BuildableTable>,
            {}

        #[doc = #try_set_discretionary_builder_trait_doc_comment]
        pub trait #try_set_field_name_discretionary_builder_trait: diesel_builders::TrySetDiscretionaryBuilder<#table_name::#field_name> + Sized
        where
            for<'a> #table_name::#field_name: diesel_builders::DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
        {
            #[inline]
            #[doc = #try_set_discretionary_builder_method_doc_comment]
            #[doc = ""]
            #[doc = " # Errors"]
            #[doc = ""]
            #[doc = "Returns an error if the column check constraints are not respected."]
            fn #try_set_field_name_builder_method_ref(
                &mut self,
                value: diesel_builders::TableBuilder<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable>
            ) -> Result<&mut Self, <<<Self as diesel::associations::HasTable>::Table as diesel_builders::TableAddition>::InsertableModel as diesel_builders::InsertableTableModel>::Error> {
                use diesel_builders::TrySetDiscretionaryBuilderExt;
                self.try_set_discretionary_builder_ref::<#table_name::#field_name>(value)
            }
            #[inline]
            #[doc = #try_set_discretionary_builder_method_doc_comment]
            #[doc = ""]
            #[doc = " # Errors"]
            #[doc = ""]
            #[doc = "Returns an error if the value cannot be converted to the column type."]
            fn #try_set_field_name_builder_method(
                self,
                value: diesel_builders::TableBuilder<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable>
            ) -> Result<Self, <<<Self as diesel::associations::HasTable>::Table as diesel_builders::TableAddition>::InsertableModel as diesel_builders::InsertableTableModel>::Error> {
                use diesel_builders::TrySetDiscretionaryBuilderExt;
                self.try_set_discretionary_builder::<#table_name::#field_name>(value)
            }
        }

        impl<T> #try_set_field_name_discretionary_builder_trait for T
        where
            T: diesel_builders::TrySetDiscretionaryBuilder<#table_name::#field_name>,
            for<'a> #table_name::#field_name: diesel_builders::DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
            {}
    }
}
