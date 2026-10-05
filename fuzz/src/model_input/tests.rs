use syn::visit::{self, Visit};

use super::{Generator, MAX_FIELDS, MAX_NESTING, generate_model};

#[derive(Default)]
struct Depth {
    types: usize,
    expressions: usize,
    max_types: usize,
    max_expressions: usize,
}

impl<'ast> Visit<'ast> for Depth {
    fn visit_type(&mut self, ty: &'ast syn::Type) {
        self.types += 1;
        self.max_types = self.max_types.max(self.types);
        visit::visit_type(self, ty);
        self.types -= 1;
    }

    fn visit_expr(&mut self, expr: &'ast syn::Expr) {
        self.expressions += 1;
        self.max_expressions = self.max_expressions.max(self.expressions);
        visit::visit_expr(self, expr);
        self.expressions -= 1;
    }
}

#[test]
fn input_length_cannot_increase_type_or_expression_nesting() {
    for byte in 0..=u8::MAX {
        let bytes = [byte; 2048];
        let mut generator = Generator::new(&bytes);
        let ty = generator.ty(0);
        let expr = generator.expr(0);
        let mut depth = Depth::default();
        depth.visit_type(&ty);
        depth.visit_expr(&expr);
        assert!(depth.max_types <= MAX_NESTING + 1, "type selector {byte}");
        assert!(depth.max_expressions <= MAX_NESTING + 1, "expression selector {byte}");
        let _: syn::Type = syn::parse2(quote::quote!(#ty)).expect("valid generated type");
        let _: syn::Expr = syn::parse2(quote::quote!(#expr)).expect("valid generated expression");
    }
}

#[test]
fn reference_and_option_nesting_reach_the_budget() {
    for selector in [1, 4] {
        let mut bytes = vec![selector; MAX_NESTING];
        bytes.extend([0, 0]);
        let ty = Generator::new(&bytes).ty(0);
        let mut depth = Depth::default();
        depth.visit_type(&ty);
        assert_eq!(depth.max_types, MAX_NESTING + 1);
    }
}

#[test]
fn model_width_is_bounded_independently_of_nesting() {
    let mut bytes = vec![0, 0, 0, 0, 0, u8::try_from(MAX_FIELDS).unwrap()];
    for _ in 0..MAX_FIELDS {
        bytes.extend([0, 0, 1, 1, 1, 1, 0, 0, 0]);
    }
    let model = generate_model(&bytes);
    let syn::Data::Struct(data) = &model.data else {
        panic!("expected a struct");
    };
    assert_eq!(data.fields.len(), MAX_FIELDS);
    let mut depth = Depth::default();
    depth.visit_derive_input(&model);
    assert_eq!(depth.max_types, MAX_NESTING + 1);
}

#[test]
fn composite_foreign_keys_preserve_column_associations() {
    let model = generate_model(&[
        0, 0, 0, 0, 2, 1, 2, 0, 3, 8, 1, 2, 0, 3, 2, 0, 7, 0, 0, 7, 3, 2, 0, 0, 0, 0, 0, 3, 0, 0,
        0, 0,
    ]);
    let (_, primary_key) =
        crate::table_model::attribute_parsing::extract_diesel_attributes(&model).unwrap();
    assert_eq!(
        primary_key.iter().map(ToString::to_string).collect::<Vec<_>>(),
        ["id", "parent_id"]
    );
    let attributes =
        crate::table_model::attribute_parsing::extract_table_model_attributes(&model).unwrap();
    let foreign_key = &attributes.foreign_keys[0];
    assert_eq!(foreign_key.host_columns, primary_key);
    assert_eq!(foreign_key.referenced_columns[0], syn::parse_quote!(parents::id));
    assert_eq!(foreign_key.referenced_columns[1], syn::parse_quote!(parents::parent_id));
    let tokens = crate::derive_table_model(&model).expect("accepted composite relationship");
    let _: syn::File = syn::parse2(tokens).expect("valid composite expansion");
}

#[test]
fn mixed_selectors_preserve_the_nesting_budget() {
    let mut state = 0x2b07_b2e1_5162_8f69_u64;
    for _ in 0..512 {
        let mut bytes = [0; 2048];
        for byte in &mut bytes {
            state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            *byte = state.to_le_bytes()[4];
        }
        let mut generator = Generator::new(&bytes);
        let ty = generator.ty(0);
        let expr = generator.expr(0);
        let mut depth = Depth::default();
        depth.visit_type(&ty);
        depth.visit_expr(&expr);
        assert!(depth.max_types <= MAX_NESTING + 1);
        assert!(depth.max_expressions <= MAX_NESTING + 1);
    }
}
