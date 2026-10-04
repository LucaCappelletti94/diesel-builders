//! Submodule providing the `SetBuilder` trait.

use diesel::Table;
use tuplities::prelude::NestedTupleInto;

use crate::{
    BuildableTable, DiscretionarySameAsIndex, ForeignPrimaryKey, GetColumnExt, GetNestedColumns,
    HasTableExt, MandatorySameAsIndex, NestedColumns, SetColumn, SetNestedColumns, TableBuilder,
    TableExt, TrySetNestedColumns, TypedColumn, TypedNestedTuple, ValidateNestedColumns,
};

/// Trait for setting a mandatory triangular builder relationship.
///
/// Mandatory relationships require that a related record is created whenever
/// the main record is created. This ensures referential integrity in triangular
/// dependencies where a child table references both a parent and a side table.
///
/// # Type Parameters
///
/// * `Key`: The foreign key relationship defining the mandatory link
pub trait SetMandatoryBuilder<Key: MandatorySameAsIndex<ReferencedTable: BuildableTable>> {
    /// Sets the mandatory builder for the specified relationship.
    ///
    /// # Examples
    ///
    /// Attach the mandatory builder, complete the discretionary link, insert,
    /// and check the relationship key.
    ///
    /// ```rust
    /// # include!("doctest_setup.rs");
    /// # use schema::*;
    /// # use diesel_builders::SetMandatoryBuilder;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut conn = connection()?;
    /// let mut builder = children::table::builder();
    /// <TableBuilder<children::table> as SetMandatoryBuilder<children::mandatory_id>>::set_mandatory_builder(
    ///     &mut builder,
    ///     sides::table::builder(),
    /// );
    /// let builder = builder.discretionary(sides::table::builder()).child_label("short");
    /// let child = builder.insert(&mut conn)?;
    /// let mandatory: Side = child.mandatory(&mut conn)?;
    /// assert_eq!(mandatory.get_column::<sides::parent_id>(), child.get_column::<children::id>());
    /// assert_eq!(child.get_column::<children::child_label>(), "short".to_string());
    /// # Ok(())
    /// # }
    /// ```
    fn set_mandatory_builder(&mut self, builder: TableBuilder<Key::ReferencedTable>) -> &mut Self;
}

/// Trait for setting a discretionary triangular builder relationship.
///
/// Discretionary relationships allow optional related records. You can either
/// provide a new builder to create a related record, or reference an existing
/// model that was created previously.
///
/// # Type Parameters
///
/// * `Key`: The foreign key relationship defining the discretionary link
pub trait SetDiscretionaryBuilder<Key: DiscretionarySameAsIndex<ReferencedTable: BuildableTable>> {
    /// Sets the discretionary builder for the specified relationship.
    ///
    /// # Examples
    ///
    /// Attach the discretionary builder, complete the mandatory link, insert,
    /// and check the relationship key.
    ///
    /// ```rust
    /// # include!("doctest_setup.rs");
    /// # use schema::*;
    /// # use diesel_builders::SetDiscretionaryBuilder;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut conn = connection()?;
    /// let mut builder = children::table::builder();
    /// <TableBuilder<children::table> as SetDiscretionaryBuilder<children::discretionary_id>>::set_discretionary_builder(
    ///     &mut builder,
    ///     sides::table::builder(),
    /// );
    /// let builder = builder.mandatory(sides::table::builder()).child_label("short");
    /// let child = builder.insert(&mut conn)?;
    /// let discretionary: Side = child.discretionary(&mut conn)?;
    /// assert_eq!(discretionary.get_column::<sides::parent_id>(), child.get_column::<children::id>());
    /// # Ok(())
    /// # }
    /// ```
    fn set_discretionary_builder(
        &mut self,
        builder: TableBuilder<Key::ReferencedTable>,
    ) -> &mut Self;
}

/// Trait for setting a discretionary relationship using an existing model.
///
/// This allows linking to an existing record in a discretionary relationship
/// rather than creating a new one.
///
/// # Type Parameters
///
/// * `Key`: The foreign key relationship defining the discretionary link
pub trait SetDiscretionaryModel<Key: DiscretionarySameAsIndex> {
    /// Sets the relationship to reference an existing model.
    ///
    /// # Examples
    ///
    /// Reference an existing side model instead of creating a new one.
    ///
    /// ```rust
    /// # include!("doctest_setup.rs");
    /// # use schema::*;
    /// # use diesel_builders::SetDiscretionaryModel;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut conn = connection()?;
    /// # let parent = parents::table::builder().label("linked").insert(&mut conn)?;
    /// # let side = sides::table::builder()
    /// #     .parent_id(parent.get_column::<parents::id>())
    /// #     .label("linked")
    /// #     .insert(&mut conn)?;
    /// let mut builder = children::table::builder()
    ///     .mandatory(
    ///         sides::table::builder()
    ///             .parent_id(parent.get_column::<parents::id>())
    ///             .label("linked"),
    ///     )
    ///     .child_label("linked");
    /// <TableBuilder<children::table> as SetDiscretionaryModel<children::discretionary_id>>::set_discretionary_model(
    ///     &mut builder,
    ///     &side,
    /// );
    /// let child = builder.insert(&mut conn)?;
    /// let discretionary: Side = child.discretionary(&mut conn)?;
    /// assert_eq!(discretionary.get_column::<sides::id>(), side.get_column::<sides::id>());
    /// # Ok(())
    /// # }
    /// ```
    fn set_discretionary_model(
        &mut self,
        model: &<Key::ReferencedTable as TableExt>::Model,
    ) -> &mut Self;
}

impl<C, T> SetDiscretionaryModel<C> for T
where
    C: DiscretionarySameAsIndex,
    Self: SetNestedColumns<C::NestedHostColumns> + SetColumn<C>,
    <<C as ForeignPrimaryKey>::ReferencedTable as TableExt>::Model:
        GetNestedColumns<C::NestedForeignColumns>,
{
    #[inline]
    fn set_discretionary_model(
        &mut self,
        model: &<<C as ForeignPrimaryKey>::ReferencedTable as TableExt>::Model,
    ) -> &mut Self {
        let primary_key = model.get_column::<<C::ReferencedTable as Table>::PrimaryKey>();
        <Self as SetColumn<C>>::set_column(self, primary_key);
        let columns = model.get_nested_columns();
        let converted_columns = columns.nested_tuple_into();
        self.set_nested_columns(converted_columns)
    }
}

/// Trait attempting to set a specific Diesel column, which may fail.
///
/// Extends [`HasTableExt`].
pub trait TrySetMandatoryBuilder<Key: MandatorySameAsIndex<ReferencedTable: BuildableTable>>:
    HasTableExt
{
    /// Attempt to set the value of the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the column cannot be set.
    ///
    /// # Examples
    ///
    /// A rejected attachment leaves the builder's prior state intact.
    ///
    /// ```rust
    /// # include!("doctest_setup.rs");
    /// # use schema::*;
    /// # use diesel_builders::TrySetMandatoryBuilder;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut builder = checked_children::table::builder();
    /// let saved = builder.clone();
    /// let result = <TableBuilder<checked_children::table> as TrySetMandatoryBuilder<
    ///     checked_children::mandatory_id,
    /// >>::try_set_mandatory_builder(
    ///     &mut builder, checked_sides::table::builder().try_label("deny")?
    /// );
    /// assert!(matches!(result, Err(ValidationError::ReservedLabel)));
    /// assert_eq!(builder, saved);
    /// # Ok(())
    /// # }
    /// ```
    fn try_set_mandatory_builder(
        &mut self,
        builder: TableBuilder<<Key as ForeignPrimaryKey>::ReferencedTable>,
    ) -> Result<&mut Self, <Self::Table as TableExt>::Error>;
}

/// Trait attempting to set a specific Diesel column, which may fail.
///
/// Extends [`HasTableExt`].
pub trait TrySetDiscretionaryBuilder<Key: DiscretionarySameAsIndex<ReferencedTable: BuildableTable>>:
    HasTableExt
{
    /// Attempt to set the value of the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the column cannot be set.
    ///
    /// # Examples
    ///
    /// A rejected attachment leaves the builder's prior state intact.
    ///
    /// ```rust
    /// # include!("doctest_setup.rs");
    /// # use schema::*;
    /// # use diesel_builders::TrySetDiscretionaryBuilder;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut builder = checked_children::table::builder();
    /// let saved = builder.clone();
    /// let result = <TableBuilder<checked_children::table> as TrySetDiscretionaryBuilder<
    ///     checked_children::discretionary_id,
    /// >>::try_set_discretionary_builder(
    ///     &mut builder,
    ///     checked_sides::table::builder().try_label("deny")?,
    /// );
    /// assert!(matches!(result, Err(ValidationError::ReservedLabel)));
    /// assert_eq!(builder, saved);
    /// # Ok(())
    /// # }
    /// ```
    fn try_set_discretionary_builder(
        &mut self,
        builder: TableBuilder<Key::ReferencedTable>,
    ) -> Result<&mut Self, <Self::Table as TableExt>::Error>;
}

/// Trait attempting to set a specific Diesel discretionary triangular model,
/// which may fail.
///
/// Extends [`HasTableExt`].
pub trait TrySetDiscretionaryModel<Key: DiscretionarySameAsIndex>: HasTableExt {
    /// Attempt to set the values associated to the provided model.
    ///
    /// # Errors
    ///
    /// Returns an error if the model cannot be set.
    ///
    /// # Examples
    ///
    /// Reference an existing side model instead of creating a new one.
    ///
    /// ```rust
    /// # include!("doctest_setup.rs");
    /// # use schema::*;
    /// # use diesel_builders::TrySetDiscretionaryModel;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut conn = connection()?;
    /// # let parent = checked_parents::table::try_builder()?.try_label("link")?.insert(&mut conn)?;
    /// # let side = checked_sides::table::builder()
    /// #     .parent_id(parent.get_column::<checked_parents::id>())
    /// #     .try_label("link")?
    /// #     .insert(&mut conn)?;
    /// let mut builder = checked_children::table::builder()
    ///     .try_mandatory(
    ///         checked_sides::table::builder()
    ///             .parent_id(parent.get_column::<checked_parents::id>())
    ///             .try_label("link")?,
    ///     )?
    ///     .try_child_label("link")?;
    /// <TableBuilder<checked_children::table> as TrySetDiscretionaryModel<checked_children::discretionary_id>>::try_set_discretionary_model(
    ///     &mut builder,
    ///     &side,
    /// )?;
    /// let child = builder.insert(&mut conn)?;
    /// let discretionary: CheckedSide = child.discretionary(&mut conn)?;
    /// assert_eq!(discretionary.get_column::<checked_sides::id>(), side.get_column::<checked_sides::id>());
    /// # Ok(())
    /// # }
    /// ```
    fn try_set_discretionary_model(
        &mut self,
        model: &<Key::ReferencedTable as TableExt>::Model,
    ) -> Result<&mut Self, <Self::Table as TableExt>::Error>;
}

impl<C, T> TrySetDiscretionaryModel<C> for T
where
    T: HasTableExt,
    C: DiscretionarySameAsIndex,
    (C, C::NestedHostColumns): NestedColumns<
        NestedTupleColumnType = (
            C::ColumnType,
            <C::NestedHostColumns as TypedNestedTuple>::NestedTupleColumnType,
        ),
    >,
    Self: TrySetNestedColumns<<Self::Table as TableExt>::Error, (C, C::NestedHostColumns)>
        + ValidateNestedColumns<<Self::Table as TableExt>::Error, C::NestedHostColumns>,
    <<C as ForeignPrimaryKey>::ReferencedTable as TableExt>::Model:
        GetNestedColumns<C::NestedForeignColumns>,
{
    #[inline]
    fn try_set_discretionary_model(
        &mut self,
        model: &<<C as ForeignPrimaryKey>::ReferencedTable as TableExt>::Model,
    ) -> Result<&mut Self, <Self::Table as TableExt>::Error> {
        let primary_key: C::ValueType =
            model.get_column::<<C::ReferencedTable as Table>::PrimaryKey>();
        let columns = model.get_nested_columns();
        let converted_columns = columns.nested_tuple_into();
        self.validate_nested_columns(&converted_columns)?;
        self.try_set_nested_columns((primary_key.into(), converted_columns))
    }
}

/// Emits an extension trait that lets callers pick the relationship marker at
/// the method level instead of on the trait.
///
/// The six builder/model extension traits are structurally identical within
/// each flavour, differing only in their names, key bound, argument type, and
/// whether they are fallible, so all of them are produced from this template.
macro_rules! set_builder_ext {
    (
        @infallible
        ext_trait = $ext:ident,
        supertrait = $supertrait:path,
        inner_trait = $inner:ident,
        inner_method = $inner_method:ident,
        ref_method = $ref_method:ident,
        method = $method:ident,
        key_bound = [$($key_bound:tt)+],
        $arg:ident: $arg_ty:ty,
        subject = $subject:literal,
        ref_example = $ref_example:literal,
        owned_example = $owned_example:literal,
        link = $link:literal $(,)?
    ) => {
        #[doc = concat!(
            "Extension trait for [`", $link, "`] that allows specifying the column at the method level."
        )]
        pub trait $ext: $supertrait {
            #[doc = concat!("Sets the ", $subject, " for the specified column.\n\n", $ref_example)]
            #[inline]
            fn $ref_method<Key>(&mut self, $arg: $arg_ty) -> &mut Self
            where
                Key: $($key_bound)+,
                Self: $inner<Key>,
            {
                <Self as $inner<Key>>::$inner_method(self, $arg)
            }

            #[doc = concat!(
                "Sets the ", $subject, " for the specified column, consuming and \
                 returning `self` for fluent chaining.\n\n", $owned_example
            )]
            #[inline]
            #[must_use]
            fn $method<Key>(mut self, $arg: $arg_ty) -> Self
            where
                Key: $($key_bound)+,
                Self: $inner<Key>,
            {
                self.$ref_method::<Key>($arg);
                self
            }
        }

        impl<T: $supertrait> $ext for T {}
    };
    (
        @fallible
        ext_trait = $ext:ident,
        supertrait = $supertrait:path,
        inner_trait = $inner:ident,
        inner_method = $inner_method:ident,
        ref_method = $ref_method:ident,
        method = $method:ident,
        key_bound = [$($key_bound:tt)+],
        consume_bound = [$($consume_bound:tt)*],
        $arg:ident: $arg_ty:ty,
        subject = $subject:literal,
        ref_example = $ref_example:literal,
        owned_example = $owned_example:literal,
        link = $link:literal $(,)?
    ) => {
        #[doc = concat!(
            "Extension trait for [`", $link, "`] that allows specifying the column at the method level."
        )]
        pub trait $ext: $supertrait {
            #[doc = concat!("Attempts to set the ", $subject, " for the specified column.")]
            #[doc = ""]
            #[doc = "# Errors"]
            #[doc = ""]
            #[doc = concat!("Returns an error if the ", $subject, " cannot be set.\n\n", $ref_example)]
            #[inline]
            fn $ref_method<Key>(
                &mut self,
                $arg: $arg_ty,
            ) -> Result<&mut Self, <Self::Table as TableExt>::Error>
            where
                Key: $($key_bound)+,
                Self: $inner<Key>,
            {
                <Self as $inner<Key>>::$inner_method(self, $arg)
            }

            #[doc = concat!(
                "Attempts to set the ", $subject, " for the specified column, consuming \
                 and returning `self` for fluent chaining."
            )]
            #[doc = ""]
            #[doc = "# Errors"]
            #[doc = ""]
            #[doc = concat!("Returns an error if the ", $subject, " cannot be set.\n\n", $owned_example)]
            #[inline]
            fn $method<Key>(
                mut self,
                $arg: $arg_ty,
            ) -> Result<Self, <Self::Table as TableExt>::Error>
            where
                Key: $($key_bound)+,
                Self: $inner<Key> $($consume_bound)*,
            {
                self.$ref_method::<Key>($arg)?;
                Ok(self)
            }
        }

        impl<T: $supertrait> $ext for T {}
    };
}

set_builder_ext! {
    @infallible
    ext_trait = SetMandatoryBuilderExt,
    supertrait = Sized,
    inner_trait = SetMandatoryBuilder,
    inner_method = set_mandatory_builder,
    ref_method = set_mandatory_builder_ref,
    method = set_mandatory_builder,
    key_bound = [MandatorySameAsIndex<ReferencedTable: BuildableTable>],
    builder: TableBuilder<Key::ReferencedTable>,
    subject = "mandatory builder",
    ref_example = r###"# Examples

Attach the mandatory builder by reference and check the relationship key.

```rust
# include!("doctest_setup.rs");
# use schema::*;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut conn = connection()?;
let mut builder = children::table::builder();
builder.set_mandatory_builder_ref::<children::mandatory_id>(sides::table::builder());
let builder = builder.discretionary(sides::table::builder()).child_label("short");
let child = builder.insert(&mut conn)?;
let mandatory: Side = child.mandatory(&mut conn)?;
assert_eq!(mandatory.get_column::<sides::parent_id>(), child.get_column::<children::id>());
# Ok(())
# }
```"###,
    owned_example = r###"# Examples

Attach the mandatory builder, consuming it for fluent chaining.

```rust
# include!("doctest_setup.rs");
# use schema::*;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut conn = connection()?;
let builder = children::table::builder()
    .set_mandatory_builder::<children::mandatory_id>(sides::table::builder())
    .discretionary(sides::table::builder())
    .child_label("short");
let child = builder.insert(&mut conn)?;
let mandatory: Side = child.mandatory(&mut conn)?;
assert_eq!(mandatory.get_column::<sides::parent_id>(), child.get_column::<children::id>());
# Ok(())
# }
```"###,
    link = "SetMandatoryBuilder",
}

set_builder_ext! {
    @infallible
    ext_trait = SetDiscretionaryBuilderExt,
    supertrait = Sized,
    inner_trait = SetDiscretionaryBuilder,
    inner_method = set_discretionary_builder,
    ref_method = set_discretionary_builder_ref,
    method = set_discretionary_builder,
    key_bound = [DiscretionarySameAsIndex<ReferencedTable: BuildableTable>],
    builder: TableBuilder<Key::ReferencedTable>,
    subject = "discretionary builder",
    ref_example = r###"# Examples

Attach the discretionary builder by reference and check the relationship key.

```rust
# include!("doctest_setup.rs");
# use schema::*;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut conn = connection()?;
let mut builder = children::table::builder();
builder.set_discretionary_builder_ref::<children::discretionary_id>(sides::table::builder());
let builder = builder.mandatory(sides::table::builder()).child_label("short");
let child = builder.insert(&mut conn)?;
let discretionary: Side = child.discretionary(&mut conn)?;
assert_eq!(discretionary.get_column::<sides::parent_id>(), child.get_column::<children::id>());
# Ok(())
# }
```"###,
    owned_example = r###"# Examples

Attach the discretionary builder, consuming it for fluent chaining.

```rust
# include!("doctest_setup.rs");
# use schema::*;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut conn = connection()?;
let builder = children::table::builder()
    .set_discretionary_builder::<children::discretionary_id>(sides::table::builder())
    .mandatory(sides::table::builder())
    .child_label("short");
let child = builder.insert(&mut conn)?;
let discretionary: Side = child.discretionary(&mut conn)?;
assert_eq!(discretionary.get_column::<sides::parent_id>(), child.get_column::<children::id>());
# Ok(())
# }
```"###,
    link = "SetDiscretionaryBuilder",
}

set_builder_ext! {
    @infallible
    ext_trait = SetDiscretionaryModelExt,
    supertrait = Sized,
    inner_trait = SetDiscretionaryModel,
    inner_method = set_discretionary_model,
    ref_method = set_discretionary_model_ref,
    method = set_discretionary_model,
    key_bound = [DiscretionarySameAsIndex],
    model: &<Key::ReferencedTable as TableExt>::Model,
    subject = "discretionary model",
    ref_example = r###"# Examples

Reference an existing side model by reference and check the link.

```rust
# include!("doctest_setup.rs");
# use schema::*;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut conn = connection()?;
# let parent = parents::table::builder().label("linked").insert(&mut conn)?;
# let side = sides::table::builder()
#     .parent_id(parent.get_column::<parents::id>())
#     .label("linked")
#     .insert(&mut conn)?;
let mut builder = children::table::builder()
    .mandatory(
        sides::table::builder()
            .parent_id(parent.get_column::<parents::id>())
            .label("linked"),
    )
    .child_label("linked");
builder.set_discretionary_model_ref::<children::discretionary_id>(&side);
let child = builder.insert(&mut conn)?;
let discretionary: Side = child.discretionary(&mut conn)?;
assert_eq!(discretionary.get_column::<sides::id>(), side.get_column::<sides::id>());
# Ok(())
# }
```"###,
    owned_example = r###"# Examples

Reference an existing side model, consuming it for fluent chaining.

```rust
# include!("doctest_setup.rs");
# use schema::*;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut conn = connection()?;
# let parent = parents::table::builder().label("linked").insert(&mut conn)?;
# let side = sides::table::builder()
#     .parent_id(parent.get_column::<parents::id>())
#     .label("linked")
#     .insert(&mut conn)?;
let builder = children::table::builder()
    .mandatory(
        sides::table::builder()
            .parent_id(parent.get_column::<parents::id>())
            .label("linked"),
    )
    .set_discretionary_model::<children::discretionary_id>(&side)
    .child_label("linked");
let child = builder.insert(&mut conn)?;
let discretionary: Side = child.discretionary(&mut conn)?;
assert_eq!(discretionary.get_column::<sides::id>(), side.get_column::<sides::id>());
# Ok(())
# }
```"###,
    link = "SetDiscretionaryModel",
}

set_builder_ext! {
    @fallible
    ext_trait = TrySetMandatoryBuilderExt,
    supertrait = HasTableExt,
    inner_trait = TrySetMandatoryBuilder,
    inner_method = try_set_mandatory_builder,
    ref_method = try_set_mandatory_builder_ref,
    method = try_set_mandatory_builder,
    key_bound = [MandatorySameAsIndex<ReferencedTable: BuildableTable>],
    consume_bound = [+ Sized],
    builder: TableBuilder<Key::ReferencedTable>,
    subject = "mandatory builder",
    ref_example = r###"# Examples

A rejected attachment leaves the builder's prior state intact.

```rust
# include!("doctest_setup.rs");
# use schema::*;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let mut builder = checked_children::table::builder();
let saved = builder.clone();
let result = builder.try_set_mandatory_builder_ref::<checked_children::mandatory_id>(
    checked_sides::table::builder().try_label("deny")?,
);
assert!(matches!(result, Err(ValidationError::ReservedLabel)));
assert_eq!(builder, saved);
# Ok(())
# }
```"###,
    owned_example = r###"# Examples

Attach the mandatory builder and complete the graph, then insert.

```rust
# include!("doctest_setup.rs");
# use schema::*;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut conn = connection()?;
let builder = checked_children::table::builder()
    .try_set_mandatory_builder::<checked_children::mandatory_id>(checked_sides::table::builder())?
    .try_discretionary(checked_sides::table::builder())?
    .try_child_label("short")?;
let child = builder.insert(&mut conn)?;
let mandatory: CheckedSide = child.mandatory(&mut conn)?;
assert_eq!(mandatory.get_column::<checked_sides::parent_id>(), child.get_column::<checked_children::id>());
# Ok(())
# }
```"###,
    link = "TrySetMandatoryBuilder",
}

set_builder_ext! {
    @fallible
    ext_trait = TrySetDiscretionaryBuilderExt,
    supertrait = HasTableExt,
    inner_trait = TrySetDiscretionaryBuilder,
    inner_method = try_set_discretionary_builder,
    ref_method = try_set_discretionary_builder_ref,
    method = try_set_discretionary_builder,
    key_bound = [DiscretionarySameAsIndex<ReferencedTable: BuildableTable>],
    consume_bound = [+ Sized],
    builder: TableBuilder<Key::ReferencedTable>,
    subject = "discretionary builder",
    ref_example = r###"# Examples

A rejected attachment leaves the builder's prior state intact.

```rust
# include!("doctest_setup.rs");
# use schema::*;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let mut builder = checked_children::table::builder();
let saved = builder.clone();
let result = builder.try_set_discretionary_builder_ref::<checked_children::discretionary_id>(
    checked_sides::table::builder().try_label("deny")?,
);
assert!(matches!(result, Err(ValidationError::ReservedLabel)));
assert_eq!(builder, saved);
# Ok(())
# }
```"###,
    owned_example = r###"# Examples

Attach the discretionary builder and complete the graph, then insert.

```rust
# include!("doctest_setup.rs");
# use schema::*;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut conn = connection()?;
let builder = checked_children::table::builder()
    .try_mandatory(checked_sides::table::builder())?
    .try_set_discretionary_builder::<checked_children::discretionary_id>(checked_sides::table::builder())?
    .try_child_label("short")?;
let child = builder.insert(&mut conn)?;
let discretionary: CheckedSide = child.discretionary(&mut conn)?;
assert_eq!(discretionary.get_column::<checked_sides::parent_id>(), child.get_column::<checked_children::id>());
# Ok(())
# }
```"###,
    link = "TrySetDiscretionaryBuilder",
}

set_builder_ext! {
    @fallible
    ext_trait = TrySetDiscretionaryModelExt,
    supertrait = Sized,
    inner_trait = TrySetDiscretionaryModel,
    inner_method = try_set_discretionary_model,
    ref_method = try_set_discretionary_model_ref,
    method = try_set_discretionary_model,
    key_bound = [DiscretionarySameAsIndex],
    consume_bound = [],
    model: &<Key::ReferencedTable as TableExt>::Model,
    subject = "discretionary model",
    ref_example = r###"# Examples

Reference an existing side model by reference and check the link.

```rust
# include!("doctest_setup.rs");
# use schema::*;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut conn = connection()?;
# let parent = checked_parents::table::try_builder()?.try_label("link")?.insert(&mut conn)?;
# let side = checked_sides::table::builder()
#     .parent_id(parent.get_column::<checked_parents::id>())
#     .try_label("link")?
#     .insert(&mut conn)?;
let mut builder = checked_children::table::builder()
    .try_mandatory(
        checked_sides::table::builder()
            .parent_id(parent.get_column::<checked_parents::id>())
            .try_label("link")?,
    )?
    .try_child_label("link")?;
builder.try_set_discretionary_model_ref::<checked_children::discretionary_id>(&side)?;
let child = builder.insert(&mut conn)?;
let discretionary: CheckedSide = child.discretionary(&mut conn)?;
assert_eq!(discretionary.get_column::<checked_sides::id>(), side.get_column::<checked_sides::id>());
# Ok(())
# }
```"###,
    owned_example = r###"# Examples

Reference an existing side model in a fluent chain.

```rust
# include!("doctest_setup.rs");
# use schema::*;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut conn = connection()?;
# let parent = checked_parents::table::try_builder()?.try_label("link")?.insert(&mut conn)?;
# let side = checked_sides::table::builder()
#     .parent_id(parent.get_column::<checked_parents::id>())
#     .try_label("link")?
#     .insert(&mut conn)?;
let builder = checked_children::table::builder()
    .try_mandatory(
        checked_sides::table::builder()
            .parent_id(parent.get_column::<checked_parents::id>())
            .try_label("link")?,
    )?
    .try_set_discretionary_model::<checked_children::discretionary_id>(&side)?
    .try_child_label("link")?;
let child = builder.insert(&mut conn)?;
let discretionary: CheckedSide = child.discretionary(&mut conn)?;
assert_eq!(discretionary.get_column::<checked_sides::id>(), side.get_column::<checked_sides::id>());
# Ok(())
# }
```"###,
    link = "TrySetDiscretionaryModel",
}

/// Trait to try set a column in a mandatory same-as relationship.
pub trait TrySetMandatorySameAsColumn<
    Key: MandatorySameAsIndex,
    C: TypedColumn<Table = Key::ReferencedTable>,
>
{
    /// The associated error type for the operation.
    type Error;

    /// Updates a column in an attached mandatory builder.
    ///
    /// # Errors
    ///
    /// Returns the target column's validation error.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::{
    ///     SetDiscretionaryBuilder, SetMandatoryBuilder, TableBuilderBundle, TrySetColumn,
    ///     TrySetMandatorySameAsColumn,
    /// };
    /// use schema::*;
    ///
    /// let mut parent = TableBuilderBundle::<checked_parents::table>::empty();
    /// TrySetColumn::<checked_parents::label>::try_set_column(&mut parent, "next")?;
    /// let mut child = TableBuilderBundle::<checked_children::table>::empty();
    /// TrySetColumn::<checked_children::child_label>::try_set_column(&mut child, "next")?;
    /// SetMandatoryBuilder::<checked_children::mandatory_id>::set_mandatory_builder(
    ///     &mut child,
    ///     checked_sides::table::builder().try_label("old")?,
    /// );
    /// SetDiscretionaryBuilder::<checked_children::discretionary_id>::set_discretionary_builder(
    ///     &mut child,
    ///     checked_sides::table::builder().try_label("next")?,
    /// );
    /// type Bundle = TableBuilderBundle<checked_children::table>;
    /// <Bundle as TrySetMandatorySameAsColumn<
    ///     checked_children::mandatory_id,
    ///     checked_sides::label,
    /// >>::try_set_mandatory_same_as_column(&mut child, "next")?;
    /// assert_eq!(
    ///     <Bundle as TrySetMandatorySameAsColumn<
    ///         checked_children::mandatory_id,
    ///         checked_sides::label,
    ///     >>::try_set_mandatory_same_as_column(&mut child, "longer")
    ///     .err(),
    ///     Some(ValidationError::LabelTooLong),
    /// );
    /// let mut conn = connection()?;
    /// let row = TableBuilder::<checked_children::table>::from_bundles((parent, (child,)))
    ///     .insert(&mut conn)?;
    /// assert_eq!(row.mandatory(&mut conn)?.label, "next");
    /// assert_eq!(row.discretionary(&mut conn)?.label, "next");
    /// # Ok(())
    /// # }
    /// ```
    fn try_set_mandatory_same_as_column(
        &mut self,
        value: impl Into<C::ColumnType>,
    ) -> Result<&mut Self, Self::Error>;
}

/// Trait to try set a column in a discretionary same-as relationship.
pub trait TrySetDiscretionarySameAsColumn<
    Key: DiscretionarySameAsIndex,
    C: TypedColumn<Table = Key::ReferencedTable>,
>
{
    /// The associated error type for the operation.
    type Error;

    /// Updates a column in an attached discretionary builder.
    ///
    /// # Errors
    ///
    /// Returns the target column's validation error.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::{
    ///     SetDiscretionaryBuilder, SetMandatoryBuilder, TableBuilderBundle, TrySetColumn,
    ///     TrySetDiscretionarySameAsColumn,
    /// };
    /// use schema::*;
    ///
    /// let mut parent = TableBuilderBundle::<checked_parents::table>::empty();
    /// TrySetColumn::<checked_parents::label>::try_set_column(&mut parent, "next")?;
    /// let mut child = TableBuilderBundle::<checked_children::table>::empty();
    /// TrySetColumn::<checked_children::child_label>::try_set_column(&mut child, "next")?;
    /// SetMandatoryBuilder::<checked_children::mandatory_id>::set_mandatory_builder(
    ///     &mut child,
    ///     checked_sides::table::builder().try_label("next")?,
    /// );
    /// SetDiscretionaryBuilder::<checked_children::discretionary_id>::set_discretionary_builder(
    ///     &mut child,
    ///     checked_sides::table::builder().try_label("old")?,
    /// );
    /// type Bundle = TableBuilderBundle<checked_children::table>;
    /// <Bundle as TrySetDiscretionarySameAsColumn<
    ///     checked_children::discretionary_id,
    ///     checked_sides::label,
    /// >>::try_set_discretionary_same_as_column(&mut child, "next")?;
    /// assert_eq!(
    ///     <Bundle as TrySetDiscretionarySameAsColumn<
    ///         checked_children::discretionary_id,
    ///         checked_sides::label,
    ///     >>::try_set_discretionary_same_as_column(&mut child, "longer")
    ///     .err(),
    ///     Some(ValidationError::LabelTooLong),
    /// );
    /// let mut conn = connection()?;
    /// let row = TableBuilder::<checked_children::table>::from_bundles((parent, (child,)))
    ///     .insert(&mut conn)?;
    /// assert_eq!(row.mandatory(&mut conn)?.label, "next");
    /// assert_eq!(row.discretionary(&mut conn)?.label, "next");
    /// # Ok(())
    /// # }
    /// ```
    fn try_set_discretionary_same_as_column(
        &mut self,
        value: impl Into<C::ColumnType>,
    ) -> Result<&mut Self, Self::Error>;
}
