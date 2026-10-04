//! Exercises native derive and index expansion through parsed `syn` inputs.

use proc_macro2::TokenStream;
use syn::DeriveInput;

/// Parses `source` as a derive input and runs the `TableModel` expansion.
fn run_derive(source: &str) -> syn::Result<TokenStream> {
    let input: DeriveInput = syn::parse_str(source)?;
    crate::table_model::derive_table_model_impl(&input)
}

/// Parses `source` as a derive input, expecting it to parse.
fn parsed(source: &str) -> DeriveInput {
    syn::parse_str(source).expect("expected parseable derive input")
}

/// Returns the named field at `index` of a parsed derive input.
fn field_at(input: &DeriveInput, index: usize) -> &syn::Field {
    let fields = match &input.data {
        syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Named(fields), .. }) => {
            Some(&fields.named)
        }
        _ => None,
    }
    .expect("expected named fields");
    fields.get(index).expect("expected the fixture field")
}

/// Checks that the expansion declares a Diesel table.
fn declares_input_table(file: &syn::File) -> bool {
    file.items.iter().any(|item| {
        let syn::Item::Macro(macro_item) = item else {
            return false;
        };
        let segments = &macro_item.mac.path.segments;
        segments.len() == 2 && segments[0].ident == "diesel" && segments[1].ident == "table"
    })
}

/// Checks that the expansion implements `TableExt` binding the input model.
fn implements_table_ext_for_model(file: &syn::File, model: &str) -> bool {
    file.items.iter().any(|item| {
        let syn::Item::Impl(impl_item) = item else {
            return false;
        };
        let Some((_, trait_path, _)) = impl_item.trait_.as_ref() else {
            return false;
        };
        if trait_path.segments.last().is_none_or(|segment| segment.ident != "TableExt") {
            return false;
        }
        impl_item.items.iter().any(|impl_item| {
            let syn::ImplItem::Type(type_item) = impl_item else {
                return false;
            };
            if type_item.ident != "Model" {
                return false;
            }
            if let syn::Type::Path(type_path) = &type_item.ty {
                type_path.path.get_ident().is_some_and(|ident| ident == model)
            } else {
                false
            }
        })
    })
}

/// Runs the expansion on `source` and requires a parseable file that declares
/// the input table and binds the input model through `TableExt`.
fn expansion_is_valid_rust(source: &str, model: &str) {
    let tokens = run_derive(source).expect("expected accepted input to expand");
    let parsed: syn::File = syn::parse2(tokens).expect("expected expansion to be valid Rust");
    assert!(declares_input_table(&parsed), "expansion must declare the input table");
    assert!(
        implements_table_ext_for_model(&parsed, model),
        "expansion must implement `TableExt` for the input model"
    );
}

/// Requires the invalid model to be rejected while the accepted counterpart,
/// differing only in the offending condition, expands.
fn invalid_is_rejected_and_counterpart_expands(invalid: &str, accepted: &str, model: &str) {
    let input: DeriveInput = syn::parse_str(invalid).expect("expected parseable input");
    assert!(
        crate::table_model::derive_table_model_impl(&input).is_err(),
        "expected the invalid model to be rejected"
    );
    expansion_is_valid_rust(accepted, model);
}

/// Counts the marker impls generated for `trait_name` in an index expansion.
fn marker_impls(file: &syn::File, trait_name: &str) -> usize {
    file.items
        .iter()
        .filter(|item| {
            let syn::Item::Impl(impl_item) = item else {
                return false;
            };
            let Some((_, trait_path, _)) = impl_item.trait_.as_ref() else {
                return false;
            };
            trait_path.segments.last().is_some_and(|segment| segment.ident == trait_name)
        })
        .count()
}

/// The plain root model expands to a parseable table declaration and `TableExt`
/// model binding.
#[test]
fn plain_root_table_expands_to_valid_rust() {
    expansion_is_valid_rust(
        r"
struct Animal {
    id: i32,
    name: String,
    description: Option<String>,
}
",
        "Animal",
    );
}

/// The surrogate key model with a custom error type expands to a parseable
/// expansion.
#[test]
fn surrogate_key_with_error_type_expands_to_valid_rust() {
    expansion_is_valid_rust(
        r"
#[diesel(table_name = animals)]
#[table_model(error = NewAnimalError, surrogate_key)]
struct Animal {
    id: i32,
    name: String,
    description: Option<String>,
}
",
        "Animal",
    );
}

/// The inheritance model with struct and field defaults expands to a parseable
/// expansion.
#[test]
fn inheritance_with_struct_and_field_defaults_expands_to_valid_rust() {
    expansion_is_valid_rust(
        r#"
#[diesel(table_name = dogs)]
#[table_model(ancestors(animals))]
#[table_model(default(animals::description, "A generic dog"))]
struct Dog {
    id: i32,
    #[table_model(default = "Unknown")]
    breed: String,
}
"#,
        "Dog",
    );
}

/// The infallible field model with a record error type expands to a parseable
/// expansion.
#[test]
fn infallible_field_with_record_error_expands_to_valid_rust() {
    expansion_is_valid_rust(
        r"
#[table_model(error = NewCatError, record_error = NewCatError, ancestors(animals))]
#[diesel(table_name = cats)]
struct Cat {
    #[infallible]
    id: i32,
    color: String,
}
",
        "Cat",
    );
}

/// The model with a single-column explicit foreign key expands to a parseable
/// expansion.
#[test]
fn explicit_single_column_foreign_key_expands_to_valid_rust() {
    expansion_is_valid_rust(
        r"
#[table_model(surrogate_key, foreign_key(parent_id, (parent_table::id)))]
#[diesel(table_name = satellite_table)]
struct Satellite {
    id: i32,
    parent_id: i32,
    field: String,
    another_field: Option<String>,
}
",
        "Satellite",
    );
}

/// The model with a composite explicit foreign key expands to a parseable
/// expansion.
#[test]
fn composite_foreign_key_expands_to_valid_rust() {
    expansion_is_valid_rust(
        r"
#[diesel(table_name = readings)]
#[table_model(foreign_key((sensor_id, site_id), (sensors::id, sites::id)))]
struct Reading {
    id: i32,
    sensor_id: i32,
    site_id: i32,
    value: f64,
}
",
        "Reading",
    );
}

/// The keyed mandatory relations with bare suffix `same_as` expand to a
/// parseable expansion.
#[test]
fn keyed_mandatory_relations_with_bare_suffix_same_as_expand_to_valid_rust() {
    expansion_is_valid_rust(
        r"
#[diesel(table_name = children)]
#[table_model(ancestors(parents))]
struct Child {
    #[same_as(first_sides::parent_id, a_id)]
    #[same_as(second_sides::parent_id, b_id)]
    id: i32,
    #[mandatory(first_sides)]
    a_id: i32,
    #[mandatory(second_sides)]
    b_id: i32,
    note: String,
}
",
        "Child",
    );
}

/// The discretionary relation with a renamed column expands to a parseable
/// expansion.
#[test]
fn discretionary_relation_with_sql_name_expands_to_valid_rust() {
    expansion_is_valid_rust(
        r#"
#[diesel(table_name = side_with_satellite)]
struct SideWithSatellite {
    id: i32,
    #[same_as(satellite_table::parent_id)]
    #[discretionary(satellite_table)]
    satellite_id: i32,
    #[table_model(sql_name = "columns")]
    remote_column: String,
}
"#,
        "SideWithSatellite",
    );
}

/// The composite primary key model without `surrogate_key` expands to a
/// parseable expansion.
#[test]
fn composite_primary_key_without_surrogate_expands_to_valid_rust() {
    expansion_is_valid_rust(
        r"
#[diesel(table_name = grants, primary_key(user_id, role_id))]
struct Grant {
    user_id: i32,
    role_id: i32,
    scope: String,
}
",
        "Grant",
    );
}

/// The name-value `ancestors` attribute expands to a parseable expansion.
#[test]
fn name_value_ancestors_expand_to_valid_rust() {
    expansion_is_valid_rust(
        r"
#[diesel(table_name = child)]
#[table_model(ancestors = parent_table)]
struct Child {
    id: i32,
    note: String,
}
",
        "Child",
    );
}

/// The raw identifier columns expand to a parseable expansion.
#[test]
fn raw_identifier_columns_expand_to_valid_rust() {
    expansion_is_valid_rust(
        r"
#[diesel(table_name = measurements)]
struct Measurement {
    id: i32,
    r#type: String,
    r#enum: Option<i32>,
}
",
        "Measurement",
    );
}

/// The qualified `Option` and custom column types expand to a parseable
/// expansion.
#[test]
fn qualified_option_and_custom_types_expand_to_valid_rust() {
    expansion_is_valid_rust(
        r"
#[diesel(table_name = readings)]
struct Reading {
    id: i32,
    #[diesel(sql_type = Nullable<Integer>)]
    value: std::option::Option<custom::Value>,
    #[diesel(sql_type = Text)]
    labels: custom::LabelList,
}
",
        "Reading",
    );
}

/// The redundant `infallible` marker without an error type expands to a
/// parseable expansion.
#[test]
fn redundant_infallible_without_error_expands_to_valid_rust() {
    expansion_is_valid_rust(
        r"
#[diesel(table_name = sessions)]
struct Session {
    #[infallible]
    id: i32,
    token: String,
}
",
        "Session",
    );
}

/// The raw identifier model name with an explicit table expands to a parseable
/// expansion.
#[test]
fn raw_model_name_with_explicit_table_expands_to_valid_rust() {
    expansion_is_valid_rust(
        r"
#[diesel(table_name = measurements)]
struct r#type {
    id: i32,
    r#enum: i32,
}
",
        "r#type",
    );
}

/// The raw identifier model name with an inferred table expands to a parseable
/// expansion.
#[test]
fn raw_model_name_with_inferred_table_expands_to_valid_rust() {
    expansion_is_valid_rust(
        r"
struct r#type {
    id: i32,
    r#enum: i32,
}
",
        "r#type",
    );
}

/// The raw identifier `_id` explicit foreign key host expands to a parseable
/// expansion.
#[test]
fn raw_id_explicit_foreign_key_host_expands_to_valid_rust() {
    expansion_is_valid_rust(
        r"
#[diesel(table_name = satellites)]
#[table_model(foreign_key(r#satellite_id, (satellites::id)))]
struct Satellite {
    id: i32,
    r#satellite_id: i32,
    note: String,
}
",
        "Satellite",
    );
}

/// The raw identifier `_id` triangular field expands to a parseable expansion.
#[test]
fn raw_id_triangular_field_expands_to_valid_rust() {
    expansion_is_valid_rust(
        r"
#[diesel(table_name = children)]
struct Child {
    #[same_as(satellite_table::parent_id)]
    id: i32,
    #[mandatory(satellite_table)]
    r#mandatory_id: i32,
    note: String,
}
",
        "Child",
    );
}

/// A raw identifier model name with a mandatory triangular field expands to
/// a parseable expansion.
#[test]
fn raw_model_name_with_mandatory_field_expands_to_valid_rust() {
    expansion_is_valid_rust(
        r"
#[diesel(table_name = types)]
struct r#type {
    #[same_as(satellite_table::parent_id)]
    id: i32,
    #[mandatory(satellite_table)]
    mandatory_id: i32,
    note: String,
}
",
        "r#type",
    );
}

/// An empty `ancestors` list is rejected while the accepted counterpart
/// expands.
#[test]
fn empty_ancestors_list_is_rejected() {
    invalid_is_rejected_and_counterpart_expands(
        r"
#[table_model(ancestors())]
struct A {
    id: i32,
}
",
        r"
#[table_model(ancestors(animals))]
struct A {
    id: i32,
}
",
        "A",
    );
}

/// A table listing itself as `ancestor` is rejected while the accepted
/// counterpart expands.
#[test]
fn table_as_own_ancestor_is_rejected() {
    invalid_is_rejected_and_counterpart_expands(
        r"
#[diesel(table_name = animals)]
#[table_model(ancestors(animals))]
struct Animal {
    id: i32,
}
",
        r"
#[diesel(table_name = animals)]
#[table_model(ancestors(pets))]
struct Animal {
    id: i32,
}
",
        "Animal",
    );
}

/// Duplicate `ancestor` entries are rejected while the accepted counterpart
/// expands.
#[test]
fn duplicate_ancestors_are_rejected() {
    invalid_is_rejected_and_counterpart_expands(
        r"
#[table_model(ancestors(animals, animals))]
struct A {
    id: i32,
}
",
        r"
#[table_model(ancestors(animals, pets))]
struct A {
    id: i32,
}
",
        "A",
    );
}

/// A `surrogate_key` with a composite primary key is rejected while the
/// accepted counterpart expands.
#[test]
fn surrogate_key_with_composite_primary_key_is_rejected() {
    invalid_is_rejected_and_counterpart_expands(
        r"
#[diesel(primary_key(a, b))]
#[table_model(surrogate_key)]
struct C {
    a: i32,
    b: i32,
}
",
        r"
#[diesel(primary_key(a, b))]
struct C {
    a: i32,
    b: i32,
}
",
        "C",
    );
}

/// A struct without named fields is rejected while the accepted counterpart
/// expands.
#[test]
fn struct_without_named_fields_is_rejected() {
    invalid_is_rejected_and_counterpart_expands(
        "struct D(i32, String);",
        "struct D { id: i32, name: String }",
        "D",
    );
}

/// A non-struct input is rejected while the accepted counterpart expands.
#[test]
fn non_struct_input_is_rejected() {
    invalid_is_rejected_and_counterpart_expands("enum E { A, B }", "struct E { id: i32 }", "E");
}

/// A primary key column missing from the struct is rejected while the accepted
/// counterpart expands.
#[test]
fn primary_key_column_missing_from_struct_is_rejected() {
    invalid_is_rejected_and_counterpart_expands(
        r"
#[diesel(primary_key(missing))]
struct F {
    id: i32,
    x: i32,
}
",
        r"
#[diesel(primary_key(id))]
struct F {
    id: i32,
    x: i32,
}
",
        "F",
    );
}

/// A field that is both `mandatory` and `discretionary` is rejected while the
/// accepted counterpart expands.
#[test]
fn mandatory_and_discretionary_on_one_field_are_rejected() {
    invalid_is_rejected_and_counterpart_expands(
        r"
struct G {
    id: i32,
    #[mandatory(t)]
    #[discretionary(t)]
    x: i32,
}
",
        r"
struct G {
    #[same_as(t::id)]
    id: i32,
    #[discretionary(t)]
    x: i32,
}
",
        "G",
    );
}

/// Duplicate `mandatory` attributes are rejected while the accepted counterpart
/// expands.
#[test]
fn duplicate_mandatory_attributes_are_rejected() {
    invalid_is_rejected_and_counterpart_expands(
        r"
struct H {
    id: i32,
    #[mandatory(t)]
    #[mandatory(u)]
    x: i32,
}
",
        r"
struct H {
    #[same_as(t::x)]
    id: i32,
    #[mandatory(t)]
    x: i32,
}
",
        "H",
    );
}

/// Duplicate `discretionary` attributes are rejected while the accepted
/// counterpart expands.
#[test]
fn duplicate_discretionary_attributes_are_rejected() {
    invalid_is_rejected_and_counterpart_expands(
        r"
struct I {
    id: i32,
    #[discretionary(t)]
    #[discretionary(u)]
    x: i32,
}
",
        r"
struct I {
    #[same_as(t::id)]
    id: i32,
    #[discretionary(t)]
    x: i32,
}
",
        "I",
    );
}

/// Duplicate `infallible` markers are rejected while the accepted counterpart
/// expands.
#[test]
fn duplicate_infallible_markers_are_rejected() {
    invalid_is_rejected_and_counterpart_expands(
        r"
#[table_model(error = MyError)]
struct J {
    #[infallible]
    #[table_model(infallible)]
    id: i32,
}
",
        r"
#[table_model(error = MyError)]
struct J {
    #[infallible]
    id: i32,
}
",
        "J",
    );
}

/// Multiple field `default` values are rejected while the accepted counterpart
/// expands.
#[test]
fn multiple_field_defaults_are_rejected() {
    invalid_is_rejected_and_counterpart_expands(
        r#"
struct K {
    id: i32,
    #[table_model(default = "a")]
    #[table_model(default = "b")]
    x: String,
}
"#,
        r#"
struct K {
    id: i32,
    #[table_model(default = "a")]
    x: String,
}
"#,
        "K",
    );
}

/// An unsupported `diesel` field attribute is rejected while the accepted
/// counterpart expands.
#[test]
fn unsupported_diesel_field_attribute_is_rejected() {
    invalid_is_rejected_and_counterpart_expands(
        r#"
struct L {
    id: i32,
    #[diesel(column_name = "c")]
    x: i32,
}
"#,
        "struct L { id: i32, x: i32 }",
        "L",
    );
}

/// A parenless `same_as` attribute is rejected while the accepted counterpart
/// expands.
#[test]
fn parenless_same_as_is_rejected() {
    invalid_is_rejected_and_counterpart_expands(
        r"
struct M {
    id: i32,
    #[same_as]
    x: i32,
}
",
        "struct M { id: i32, x: i32 }",
        "M",
    );
}

/// A `surrogate_key` model with a `mandatory` field is rejected while the
/// accepted counterpart expands.
#[test]
fn surrogate_key_with_mandatory_field_is_rejected() {
    invalid_is_rejected_and_counterpart_expands(
        r"
#[table_model(surrogate_key)]
struct N {
    id: i32,
    #[mandatory(t)]
    t_id: i32,
}
",
        r"
struct N {
    #[same_as(t::x)]
    id: i32,
    #[mandatory(t)]
    t_id: i32,
}
",
        "N",
    );
}

/// A `mandatory` relation without a primary key `same_as` is rejected while the
/// accepted counterpart expands.
#[test]
fn mandatory_relation_without_primary_key_same_as_is_rejected() {
    invalid_is_rejected_and_counterpart_expands(
        r"
struct O {
    id: i32,
    #[mandatory(t)]
    t_id: i32,
}
",
        r"
struct O {
    #[same_as(t::x)]
    id: i32,
    #[mandatory(t)]
    t_id: i32,
}
",
        "O",
    );
}

/// A bare suffix `same_as` in the first position is rejected while the accepted
/// counterpart expands.
#[test]
fn bare_suffix_same_as_in_first_position_is_rejected() {
    invalid_is_rejected_and_counterpart_expands(
        r"
struct P {
    #[same_as(y, other::x)]
    id: i32,
    #[mandatory(t)]
    t_id: i32,
}
",
        r"
struct P {
    #[same_as(t::x, t_id)]
    id: i32,
    #[mandatory(t)]
    t_id: i32,
}
",
        "P",
    );
}

/// The tuple `same_as` attribute expands to one group per tuple member.
#[test]
fn same_as_tuple_attribute_expands_to_one_group_per_member() {
    let input = parsed("struct T { #[same_as(p::a, (p::b, p::c))] x: i32 }");
    let field = field_at(&input, 0);
    let groups = crate::table_model::attribute_parsing::extract_same_as_columns(field)
        .expect("expected the same_as groups to parse");
    let first: syn::Path = syn::parse_str("p::a").expect("expected a valid path");
    let second: syn::Path = syn::parse_str("p::b").expect("expected a valid path");
    let third: syn::Path = syn::parse_str("p::c").expect("expected a valid path");
    assert_eq!(groups, vec![vec![first.clone(), second], vec![first, third]]);
}

/// An empty tuple `same_as` attribute is rejected while a non-empty one is
/// accepted.
#[test]
fn same_as_empty_tuple_is_rejected() {
    let accepted_input = parsed("struct T { #[same_as(p::a, (p::b))] x: i32 }");
    let invalid_input = parsed("struct T { #[same_as(p::a, ())] x: i32 }");
    let accepted = field_at(&accepted_input, 0);
    let invalid = field_at(&invalid_input, 0);
    assert!(
        crate::table_model::attribute_parsing::extract_same_as_columns(invalid).is_err(),
        "expected the empty tuple to be rejected"
    );
    assert!(
        crate::table_model::attribute_parsing::extract_same_as_columns(accepted).is_ok(),
        "expected a non-empty tuple to be accepted"
    );
}

/// The explicit `foreign_key` attribute is captured with its host and
/// referenced columns.
#[test]
fn explicit_foreign_key_attribute_is_captured() {
    let input = parsed(
        "#[table_model(foreign_key((a, b), (t::x, t::y)))] struct T { id: i32, a: i32, b: i32 }",
    );
    let attributes = crate::table_model::attribute_parsing::extract_table_model_attributes(&input)
        .expect("expected the table_model attributes to parse");
    let foreign_key = &attributes.foreign_keys[0];
    let expected_host: Vec<syn::Ident> = vec![
        syn::parse_str("a").expect("expected a valid ident"),
        syn::parse_str("b").expect("expected a valid ident"),
    ];
    let expected_referenced: Vec<syn::Path> = vec![
        syn::parse_str("t::x").expect("expected a valid path"),
        syn::parse_str("t::y").expect("expected a valid path"),
    ];
    assert_eq!(foreign_key.host_columns, expected_host);
    assert_eq!(foreign_key.referenced_columns, expected_referenced);
}

/// The `sql_name` attribute is captured as its value.
#[test]
fn sql_name_attribute_is_captured() {
    let input = parsed("struct T { id: i32, #[table_model(sql_name = \"custom\")] x: i32 }");
    let field = field_at(&input, 1);
    let name = crate::table_model::attribute_parsing::extract_sql_name(field)
        .expect("expected the sql_name attribute to be captured");
    assert_eq!(name, "custom");
}

/// The index expansion emits one marker impl per column.
#[test]
fn index_expansion_emits_one_marker_impl_per_column() {
    let body: TokenStream = quote::quote! { satellite_table :: id , satellite_table :: field };
    let tokens =
        crate::index::expand_index(body, &quote::quote! { ::diesel_builders::IndexedColumn })
            .expect("expected a terminated column list to expand");
    let parsed: syn::File = syn::parse2(tokens).expect("expected valid Rust");
    assert_eq!(marker_impls(&parsed, "IndexedColumn"), 2);
}

/// The unique index expansion accepts a single column.
#[test]
fn unique_index_expansion_accepts_single_column() {
    let body: TokenStream = quote::quote! { readings :: id };
    let tokens = crate::index::expand_index(
        body,
        &quote::quote! { ::diesel_builders::UniquelyIndexedColumn },
    )
    .expect("expected a single column to expand");
    let parsed: syn::File = syn::parse2(tokens).expect("expected valid Rust");
    assert_eq!(marker_impls(&parsed, "UniquelyIndexedColumn"), 1);
}

/// A trait-bound compound type is not a column path and must be rejected
/// instead of producing an impl target Rust cannot parse.
#[test]
fn index_expansion_rejects_trait_bound_type() {
    let body: TokenStream = quote::quote! { Foo + Bar };
    assert!(
        crate::index::expand_index(body, &quote::quote! { ::diesel_builders::IndexedColumn })
            .is_err(),
        "expected a trait-bound type to be rejected as a column"
    );
}

/// A path whose only segment is a reserved keyword parses as a valid
/// `syn::Path` but cannot appear as an impl target, so the expansion
/// itself must be rejected rather than producing unparseable output.
#[test]
fn index_expansion_rejects_reserved_keyword_path() {
    let body: TokenStream = quote::quote! { try };
    assert!(
        crate::index::expand_index(body, &quote::quote! { ::diesel_builders::IndexedColumn })
            .is_err(),
        "expected a reserved-keyword path to be rejected rather than expanded to broken Rust"
    );
}

/// The index expansion accepts an empty body.
#[test]
fn index_expansion_accepts_empty_body() {
    let tokens = crate::index::expand_index(
        TokenStream::new(),
        &quote::quote! { ::diesel_builders::IndexedColumn },
    )
    .expect("expected an empty body to expand");
    let parsed: syn::File = syn::parse2(tokens).expect("expected valid Rust");
    assert_eq!(marker_impls(&parsed, "IndexedColumn"), 0);
}

/// Unseparated index columns are rejected while the accepted body expands.
#[test]
fn index_expansion_rejects_unseparated_columns() {
    let trait_path: TokenStream = quote::quote! { ::diesel_builders::IndexedColumn };
    let invalid: TokenStream = quote::quote! { a b };
    assert!(
        crate::index::expand_index(invalid, &trait_path).is_err(),
        "expected unseparated columns to be rejected"
    );
    let accepted: TokenStream = quote::quote! { a, b };
    let tokens = crate::index::expand_index(accepted, &trait_path)
        .expect("expected the accepted body to expand");
    let parsed: syn::File = syn::parse2(tokens).expect("expected valid Rust");
    assert_eq!(marker_impls(&parsed, "IndexedColumn"), 2);
}

/// An all-underscore field name stripped down to nothing must not panic
/// building its method identifier.
#[test]
fn all_underscore_field_name_expands_to_valid_rust() {
    expansion_is_valid_rust(
        r"
struct Reading {
    id: i32,
    _________________: Option<String>,
}
",
        "Reading",
    );
}

/// A field named with a bare underscore is rejected: it cannot be
/// referenced as a column path.
#[test]
fn bare_underscore_field_name_is_rejected() {
    invalid_is_rejected_and_counterpart_expands(
        r"
struct Probe {
    id: i32,
    _: Option<String>,
}
",
        r"
struct Probe {
    id: i32,
    unnamed: Option<String>,
}
",
        "Probe",
    );
}
