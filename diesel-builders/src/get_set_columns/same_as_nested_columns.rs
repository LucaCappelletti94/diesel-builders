//! Traits for fallibly and infallibly setting columns that participate in
//! mandatory or discretionary same-as relationships.
//!
//! The mandatory and discretionary variants are structurally identical, so both
//! trait families are emitted from the `impl_same_as_nested_columns` macro,
//! keyed by their same-as marker, group preparation trait, and generated
//! identifiers.
//!
//! The fallible setter delegates to the group preparation traits, so every
//! leaf of a group is prepared before any of them is applied and a later
//! rejected leaf leaves the earlier side builders untouched.

use crate::{
    OptionalRef,
    columns::NestedColumns,
    mutation::MutationContext,
    mutation_same_as::{PrepareDiscretionaryGroup, PrepareMandatoryGroup},
};

/// Emits a `TrySet…SameAsNestedColumns` / `Set…SameAsNestedColumns` trait
/// family for one same-as flavour.
///
/// The two flavours differ only in their group preparation trait and the
/// generated identifiers, so the whole family is produced from this single
/// template.
macro_rules! impl_same_as_nested_columns {
    (
        kind = $kind:literal,
        group_trait = $group_trait:ident,
        prepare_group_method = $prepare_group_method:ident,
        apply_group_method = $apply_group_method:ident,
        try_trait = $try_trait:ident,
        try_method = $try_method:ident,
        set_trait = $set_trait:ident,
        set_method = $set_method:ident,
        try_example = $try_example:literal,
        set_example = $set_example:literal $(,)?
    ) => {
        #[doc = concat!(
                    "Trait for fallibly setting columns in a ", $kind,
                    " same-as relationship."
                )]
        pub trait $try_trait<Type, Error, Keys: NestedColumns, CS: NestedColumns> {
            #[doc = concat!(
                        "Attempts to set the value of the specified columns in the ", $kind,
                        " same-as relationship."
                    )]
            ///
            /// # Errors
            ///
            #[doc = concat!(
                        "Returns an error if the column values cannot be set in the ", $kind,
                        " same-as relationship."
                    )]
            #[doc = $try_example]
            fn $try_method(&mut self, value: &impl OptionalRef<Type>) -> Result<&mut Self, Error>;
        }

        impl<Type, Error, T, Keys: NestedColumns, CS: NestedColumns>
            $try_trait<Type, Error, Keys, CS> for T
        where
            T: $group_trait<Error, Type, Keys, CS>,
        {
            #[inline]
            fn $try_method(&mut self, value: &impl OptionalRef<Type>) -> Result<&mut Self, Error> {
                let prepared = <T as $group_trait<Error, Type, Keys, CS>>::$prepare_group_method(
                    self,
                    value,
                    &MutationContext::default(),
                )?;
                <T as $group_trait<Error, Type, Keys, CS>>::$apply_group_method(self, prepared);
                Ok(self)
            }
        }

        #[doc = concat!(
                    "Trait for infallibly setting columns in a ", $kind,
                    " same-as relationship."
                )]
        pub trait $set_trait<Type, Keys: NestedColumns, CS: NestedColumns> {
            #[doc = concat!(
                        "Sets the value of the specified columns in the ", $kind,
                        " same-as relationship."
                    )]
            #[doc = $set_example]
            fn $set_method(&mut self, value: &impl OptionalRef<Type>) -> &mut Self;
        }

        impl<T, Type, Keys, CS> $set_trait<Type, Keys, CS> for T
        where
            Keys: NestedColumns,
            CS: NestedColumns,
            T: $try_trait<Type, core::convert::Infallible, Keys, CS>,
        {
            #[inline]
            fn $set_method(&mut self, value: &impl OptionalRef<Type>) -> &mut Self {
                self.$try_method(value).unwrap_or_else(|err| match err {})
            }
        }
    };
}

impl_same_as_nested_columns! {
    kind = "mandatory",
    group_trait = PrepareMandatoryGroup,
    prepare_group_method = prepare_mandatory_group,
    apply_group_method = apply_mandatory_group,
    try_trait = TrySetMandatorySameAsNestedColumns,
    try_method = try_set_mandatory_same_as_nested_columns,
    set_trait = SetMandatorySameAsNestedColumns,
    set_method = set_mandatory_same_as_nested_columns,
    try_example = r#"
```
# include!("../doctest_setup.rs");
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use diesel_builders::{
    TrySetMandatorySameAsNestedColumns, horizontal_same_as_group::HorizontalSameAsGroupExt,
};
use schema::*;

let mut conn = connection()?;
let mut builder = checked_children::table::builder()
    .try_mandatory(checked_sides::table::builder())?
    .try_discretionary(checked_sides::table::builder())?;
type Keys = <checked_children::child_label
    as HorizontalSameAsGroupExt>::NestedMandatoryHorizontalKeys;
type CS = <checked_children::child_label
    as HorizontalSameAsGroupExt>::NestedMandatoryForeignColumns;
let short = "short".to_string();
TrySetMandatorySameAsNestedColumns::<String, ValidationError, Keys, CS>::try_set_mandatory_same_as_nested_columns(
    &mut builder,
    &short,
)?;
assert_eq!(
    TrySetMandatorySameAsNestedColumns::<String, ValidationError, Keys, CS>::try_set_mandatory_same_as_nested_columns(
        &mut builder,
        &"longer".to_string(),
    )
    .err(),
    Some(ValidationError::LabelTooLong)
);
builder.try_child_label_ref("short")?;
let child = builder.insert(&mut conn)?;
assert_eq!(child.child_label, "short");
assert_eq!(child.mandatory(&mut conn)?.label, "short");
# Ok(())
# }
```
"#,
    set_example = r#"
```
# include!("../doctest_setup.rs");
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use diesel_builders::{
    SetMandatorySameAsNestedColumns, horizontal_same_as_group::HorizontalSameAsGroupExt,
};
use schema::*;

let mut conn = connection()?;
let mut builder = children::table::builder()
    .mandatory(sides::table::builder())
    .discretionary(sides::table::builder());
type Keys = <children::child_label
    as HorizontalSameAsGroupExt>::NestedMandatoryHorizontalKeys;
type CS = <children::child_label
    as HorizontalSameAsGroupExt>::NestedMandatoryForeignColumns;
let short = "short".to_string();
SetMandatorySameAsNestedColumns::<String, Keys, CS>::set_mandatory_same_as_nested_columns(
    &mut builder,
    &short,
);
builder.child_label_ref("short");
let child = builder.insert(&mut conn)?;
assert_eq!(child.child_label, "short");
assert_eq!(child.mandatory(&mut conn)?.label, "short");
# Ok(())
# }
```
"#,
}

impl_same_as_nested_columns! {
    kind = "discretionary",
    group_trait = PrepareDiscretionaryGroup,
    prepare_group_method = prepare_discretionary_group,
    apply_group_method = apply_discretionary_group,
    try_trait = TrySetDiscretionarySameAsNestedColumns,
    try_method = try_set_discretionary_same_as_nested_columns,
    set_trait = SetDiscretionarySameAsNestedColumns,
    set_method = set_discretionary_same_as_nested_columns,
    try_example = r#"
```
# include!("../doctest_setup.rs");
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use diesel_builders::{
    TrySetDiscretionarySameAsNestedColumns,
    horizontal_same_as_group::HorizontalSameAsGroupExt,
};
use schema::*;

let mut conn = connection()?;
let mut builder = checked_children::table::builder()
    .try_mandatory(checked_sides::table::builder())?
    .try_discretionary(checked_sides::table::builder())?;
type Keys = <checked_children::child_label
    as HorizontalSameAsGroupExt>::NestedDiscretionaryHorizontalKeys;
type CS = <checked_children::child_label
    as HorizontalSameAsGroupExt>::NestedDiscretionaryForeignColumns;
let short = "short".to_string();
TrySetDiscretionarySameAsNestedColumns::<String, ValidationError, Keys, CS>::try_set_discretionary_same_as_nested_columns(
    &mut builder,
    &short,
)?;
assert_eq!(
    TrySetDiscretionarySameAsNestedColumns::<String, ValidationError, Keys, CS>::try_set_discretionary_same_as_nested_columns(
        &mut builder,
        &"longer".to_string(),
    )
    .err(),
    Some(ValidationError::LabelTooLong)
);
builder.try_child_label_ref("short")?;
let child = builder.insert(&mut conn)?;
assert_eq!(child.child_label, "short");
assert_eq!(child.discretionary(&mut conn)?.parent_id, child.id);
# Ok(())
# }
```
"#,
    set_example = r#"
```
# include!("../doctest_setup.rs");
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use diesel_builders::{
    SetDiscretionarySameAsNestedColumns, horizontal_same_as_group::HorizontalSameAsGroupExt,
};
use schema::*;

let mut conn = connection()?;
let mut builder = children::table::builder()
    .mandatory(sides::table::builder())
    .discretionary(sides::table::builder());
type Keys = <children::child_label
    as HorizontalSameAsGroupExt>::NestedDiscretionaryHorizontalKeys;
type CS = <children::child_label
    as HorizontalSameAsGroupExt>::NestedDiscretionaryForeignColumns;
let short = "short".to_string();
SetDiscretionarySameAsNestedColumns::<String, Keys, CS>::set_discretionary_same_as_nested_columns(
    &mut builder,
    &short,
);
builder.child_label_ref("short");
let child = builder.insert(&mut conn)?;
assert_eq!(child.child_label, "short");
assert_eq!(child.discretionary(&mut conn)?.parent_id, child.id);
# Ok(())
# }
```
"#,
}
