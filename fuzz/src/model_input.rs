//! Generates bounded model syntax from fuzz bytes.

use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{Attribute, DeriveInput, Expr, Ident, Type};

const MAX_NESTING: usize = 4;
const MAX_FIELDS: usize = 32;
const MAX_ATTRIBUTES: usize = 8;
const MAX_COLUMNS: usize = 4;

/// Generates a model with bounded syntax depth and independently varied fields.
pub fn generate_model(bytes: &[u8]) -> DeriveInput {
    let mut generator = Generator::new(bytes);
    let ident = match generator.byte() % 5 {
        0 => format_ident!("Model"),
        1 => Ident::new_raw("type", Span::call_site()),
        2 => format_ident!("Reading"),
        3 => generator.ident(),
        _ => format_ident!("Δ"),
    };
    let visibility = generator.byte() % 2;
    let kind = generator.byte() % 5;
    let generics = match generator.byte() % 4 {
        0 => syn::Generics::default(),
        1 => syn::parse_quote!(<T>),
        2 => syn::parse_quote!(<'a>),
        _ => syn::parse_quote!(<T, const N: usize>),
    };
    let count = generator.count(MAX_ATTRIBUTES);
    let attrs = (0..count).map(|_| generator.container_attribute()).collect();
    let count = if kind == 2 { 0 } else { generator.count(MAX_FIELDS) };
    let mut fields = (0..count)
        .map(|_| {
            let ident = generator.ident();
            let vis = if generator.byte().is_multiple_of(2) {
                syn::Visibility::Inherited
            } else {
                syn::parse_quote!(pub)
            };
            let ty = generator.ty(0);
            let count = generator.count(MAX_ATTRIBUTES);
            let attrs = (0..count).map(|_| generator.field_attribute()).collect();
            syn::Field {
                attrs,
                vis,
                mutability: syn::FieldMutability::None,
                ident: Some(ident),
                colon_token: Some(Default::default()),
                ty,
            }
        })
        .collect::<Vec<_>>();
    let mut model: DeriveInput = syn::parse_quote!(struct #ident;);
    model.attrs = attrs;
    model.generics = generics;
    if visibility != 0 {
        model.vis = syn::parse_quote!(pub);
    }
    let named = |fields: Vec<syn::Field>| {
        syn::FieldsNamed { brace_token: Default::default(), named: fields.into_iter().collect() }
    };
    model.data = match kind {
        0 => {
            syn::Data::Struct(syn::DataStruct {
                struct_token: Default::default(),
                fields: syn::Fields::Named(named(fields)),
                semi_token: None,
            })
        }
        1 => {
            for field in &mut fields {
                field.ident = None;
                field.colon_token = None;
            }
            syn::Data::Struct(syn::DataStruct {
                struct_token: Default::default(),
                fields: syn::Fields::Unnamed(syn::FieldsUnnamed {
                    paren_token: Default::default(),
                    unnamed: fields.into_iter().collect(),
                }),
                semi_token: Some(Default::default()),
            })
        }
        2 => model.data,
        3 => {
            let mut variant: syn::Variant = syn::parse_quote!(Variant);
            variant.fields = syn::Fields::Named(named(fields));
            syn::Data::Enum(syn::DataEnum {
                enum_token: Default::default(),
                brace_token: Default::default(),
                variants: std::iter::once(variant).collect(),
            })
        }
        _ => {
            syn::Data::Union(syn::DataUnion {
                union_token: Default::default(),
                fields: named(fields),
            })
        }
    };
    model
}

struct Generator<'a> {
    bytes: std::slice::Iter<'a, u8>,
}

impl<'a> Generator<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes: bytes.iter() }
    }

    fn byte(&mut self) -> u8 {
        self.bytes.next().copied().unwrap_or(0)
    }

    fn count(&mut self, maximum: usize) -> usize {
        usize::from(self.byte()) % (maximum + 1)
    }

    fn ident(&mut self) -> Ident {
        const NAMES: [&str; 23] = [
            "id",
            "name",
            "description",
            "parent_id",
            "a_id",
            "b_id",
            "note",
            "parents",
            "first_sides",
            "second_sides",
            "models",
            "animals",
            "dogs",
            "children",
            "custom",
            "Value",
            "String",
            "type",
            "match",
            "_id",
            "_0",
            "_1_id",
            "Δ",
        ];
        let selector = usize::from(self.byte()) % (NAMES.len() + 1);
        if selector < NAMES.len() {
            let name = NAMES[selector];
            return if matches!(name, "type" | "match") {
                Ident::new_raw(name, Span::call_site())
            } else {
                Ident::new(name, Span::call_site())
            };
        }
        let count = self.count(31) + 1;
        let mut name = String::with_capacity(count + 1);
        const FIRST: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";
        const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_0123456789";
        name.push(char::from(FIRST[usize::from(self.byte()) % FIRST.len()]));
        for _ in 1..count {
            name.push(char::from(ALPHABET[usize::from(self.byte()) % ALPHABET.len()]));
        }
        if matches!(name.as_str(), "_" | "self" | "Self" | "super" | "crate") {
            name.push('_');
        }
        syn::parse_str(&name).unwrap_or_else(|_| Ident::new_raw(&name, Span::call_site()))
    }

    fn path(&mut self) -> syn::Path {
        let count = self.count(2) + 1;
        let segments = (0..count).map(|_| syn::PathSegment::from(self.ident())).collect();
        syn::Path { leading_colon: None, segments }
    }

    fn column(&mut self) -> syn::Path {
        let mut path = self.path();
        path.segments.push(self.ident().into());
        path
    }

    fn string(&mut self) -> syn::LitStr {
        let count = self.count(32);
        let value = (0..count).map(|_| char::from(self.byte())).collect::<String>();
        syn::LitStr::new(&value, Span::call_site())
    }

    fn leaf_type(&mut self) -> Type {
        match self.byte() % 16 {
            0 => syn::parse_quote!(i32),
            1 => syn::parse_quote!(i64),
            2 => syn::parse_quote!(u32),
            3 => syn::parse_quote!(bool),
            4 => syn::parse_quote!(String),
            5 => syn::parse_quote!(usize),
            6 => syn::parse_quote!(()),
            7 => syn::parse_quote!(T),
            8 => syn::parse_quote!(custom::Value),
            9 => {
                let path = self.path();
                syn::parse_quote!(#path)
            }
            10 => syn::parse_quote!(f64),
            11 => syn::parse_quote!(u64),
            12 => syn::parse_quote!(i16),
            13 => syn::parse_quote!(u16),
            14 => syn::parse_quote!(chrono::NaiveDateTime),
            _ => syn::parse_quote!(u8),
        }
    }

    fn ty(&mut self, depth: usize) -> Type {
        if depth == MAX_NESTING {
            return self.leaf_type();
        }
        let selector = self.byte() % 17;
        if selector == 0 {
            return self.leaf_type();
        }
        if selector == 15 {
            let path = self.path();
            return syn::parse_quote!(dyn #path);
        }
        let inner = self.ty(depth + 1);
        match selector {
            1 => syn::parse_quote!(Option<#inner>),
            2 => syn::parse_quote!(std::option::Option<#inner>),
            3 => syn::parse_quote!(Vec<#inner>),
            4 => syn::parse_quote!(&#inner),
            5 => syn::parse_quote!(*const #inner),
            6 => syn::parse_quote!([#inner]),
            7 => {
                let length = usize::from(self.byte());
                syn::parse_quote!([#inner; #length])
            }
            8 => {
                let other = self.ty(depth + 1);
                syn::parse_quote!((#inner, #other))
            }
            9 => syn::parse_quote!(fn(i32) -> #inner),
            10 => syn::parse_quote!((#inner)),
            11 => {
                let other = self.ty(depth + 1);
                syn::parse_quote!(Result<#inner, #other>)
            }
            12 => syn::parse_quote!(custom::Value<#inner>),
            13 => syn::parse_quote!(&'a mut #inner),
            14 => syn::parse_quote!(*mut #inner),
            _ => {
                let path = self.path();
                if self.byte().is_multiple_of(2) {
                    syn::parse_quote!(#path<#inner>)
                } else {
                    let other = self.ty(depth + 1);
                    syn::parse_quote!(#path<#inner, #other>)
                }
            }
        }
    }

    fn leaf_expr(&mut self) -> Expr {
        match self.byte() % 6 {
            0 => {
                let value = u64::from(self.byte());
                syn::parse_quote!(#value)
            }
            1 => {
                let value = self.string();
                syn::parse_quote!(#value)
            }
            2 => syn::parse_quote!(true),
            3 => syn::parse_quote!(false),
            4 => syn::parse_quote!(None),
            _ => {
                let path = self.path();
                syn::parse_quote!(#path)
            }
        }
    }

    fn expr(&mut self, depth: usize) -> Expr {
        if depth == MAX_NESTING {
            return self.leaf_expr();
        }
        let selector = self.byte() % 12;
        let cost = if matches!(selector, 3 | 9 | 11) { 2 } else { 1 };
        if selector == 0 || depth + cost > MAX_NESTING {
            return self.leaf_expr();
        }
        let inner = self.expr(depth + cost);
        match selector {
            1 => syn::parse_quote!(Some(#inner)),
            2 => syn::parse_quote!(custom::value(#inner)),
            3 => syn::parse_quote!((#inner).clone()),
            4 => syn::parse_quote!(&#inner),
            5 => syn::parse_quote!(-#inner),
            6 => {
                let other = self.expr(depth + cost);
                syn::parse_quote!((#inner, #other))
            }
            7 => {
                let other = self.expr(depth + cost);
                syn::parse_quote!([#inner, #other])
            }
            8 => syn::parse_quote!({ #inner }),
            9 => {
                let other = self.expr(depth + cost);
                syn::parse_quote!((#inner) + (#other))
            }
            10 => syn::parse_quote!(|value| #inner),
            _ => {
                let ty = self.leaf_type();
                syn::parse_quote!((#inner) as #ty)
            }
        }
    }

    fn container_attribute(&mut self) -> Attribute {
        match self.byte() % 16 {
            0 => {
                let table = self.ident();
                syn::parse_quote!(#[diesel(table_name = #table)])
            }
            1 => {
                let count = self.count(MAX_COLUMNS);
                let columns = (0..count).map(|_| self.ident()).collect::<Vec<_>>();
                syn::parse_quote!(#[diesel(primary_key(#(#columns),*))])
            }
            2 => syn::parse_quote!(#[table_model(surrogate_key)]),
            3 => {
                let ty = self.ty(0);
                syn::parse_quote!(#[table_model(error = #ty)])
            }
            4 => {
                let ty = self.ty(0);
                syn::parse_quote!(#[table_model(record_error = #ty)])
            }
            5 => {
                let count = self.count(MAX_COLUMNS);
                let paths = (0..count).map(|_| self.path()).collect::<Vec<_>>();
                syn::parse_quote!(#[table_model(ancestors(#(#paths),*))])
            }
            6 => {
                let path = self.path();
                syn::parse_quote!(#[table_model(ancestors = #path)])
            }
            7 => {
                let column = self.column();
                let value = self.expr(0);
                syn::parse_quote!(#[table_model(default(#column, #value))])
            }
            8 => {
                let tuple = !self.byte().is_multiple_of(2);
                let count = self.count(MAX_COLUMNS);
                let hosts = (0..count).map(|_| self.ident()).collect::<Vec<_>>();
                let count = self.count(MAX_COLUMNS);
                let targets = (0..count).map(|_| self.column()).collect::<Vec<_>>();
                let hosts = if tuple { quote!((#(#hosts),*)) } else { quote!(#(#hosts),*) };
                syn::parse_quote!(#[table_model(foreign_key(#hosts, (#(#targets),*)))])
            }
            9 => syn::parse_quote!(#[table_model(unknown)]),
            10 => {
                let value = self.expr(0);
                syn::parse_quote!(#[diesel(unknown = #value)])
            }
            11 => syn::parse_quote!(#[table_model(error)]),
            12 => syn::parse_quote!(#[table_model(default)]),
            13 => syn::parse_quote!(#[table_model(foreign_key)]),
            14 => syn::parse_quote!(#[table_model()]),
            _ => {
                let value = self.string();
                syn::parse_quote!(#[table_model(insertable = #value)])
            }
        }
    }

    fn field_attribute(&mut self) -> Attribute {
        match self.byte() % 14 {
            0 => {
                let count = self.count(MAX_COLUMNS);
                let columns = (0..count).map(|_| self.column()).collect::<Vec<_>>();
                let key = if self.byte().is_multiple_of(2) {
                    quote!()
                } else {
                    let key = self.ident();
                    quote!(, #key)
                };
                syn::parse_quote!(#[same_as(#(#columns),* #key)])
            }
            1 => {
                let table = self.path();
                syn::parse_quote!(#[mandatory(#table)])
            }
            2 => {
                let table = self.path();
                syn::parse_quote!(#[discretionary(#table)])
            }
            3 => syn::parse_quote!(#[infallible]),
            4 => syn::parse_quote!(#[table_model(infallible)]),
            5 => {
                let value = self.expr(0);
                syn::parse_quote!(#[table_model(default = #value)])
            }
            6 => {
                let value = self.string();
                syn::parse_quote!(#[diesel(sql_name = #value)])
            }
            7 => syn::parse_quote!(#[mandatory]),
            8 => syn::parse_quote!(#[discretionary()]),
            9 => syn::parse_quote!(#[same_as()]),
            10 => syn::parse_quote!(#[table_model(default)]),
            11 => syn::parse_quote!(#[unknown]),
            12 => {
                let value = self.string();
                syn::parse_quote!(#[table_model(sql_name = #value)])
            }
            _ => {
                let ty: syn::Path = match self.byte() % 8 {
                    0 => syn::parse_quote!(Integer),
                    1 => syn::parse_quote!(BigInt),
                    2 => syn::parse_quote!(Text),
                    3 => syn::parse_quote!(Bool),
                    4 => syn::parse_quote!(Double),
                    5 => syn::parse_quote!(Binary),
                    6 => syn::parse_quote!(Nullable<Integer>),
                    _ => syn::parse_quote!(Array<Text>),
                };
                syn::parse_quote!(#[diesel(sql_type = #ty)])
            }
        }
    }
}

#[cfg(test)]
mod tests;
