//! Submodule providing the `SetBuilder` trait.

use diesel::Table;
use tuplities::prelude::NestedTupleInto;

use crate::{
    BuildableTable, DiscretionarySameAsIndex, ForeignPrimaryKey, GetColumnExt, GetNestedColumns,
    HasTableExt, MandatorySameAsIndex, SetColumn, SetNestedColumns, TableBuilder, TableExt,
    TrySetColumn, TrySetNestedColumns, TypedColumn, ValidateColumn, ValidateNestedColumns,
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
    /// This associates a builder for the related table that will be created
    /// atomically with the main record.
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
    /// This associates a builder for the related table that will be created
    /// along with the main record.
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
    /// This copies the relevant field values from the existing model
    /// to establish the relationship.
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
    fn try_set_discretionary_model(
        &mut self,
        model: &<Key::ReferencedTable as TableExt>::Model,
    ) -> Result<&mut Self, <Self::Table as TableExt>::Error>;
}

impl<C, T> TrySetDiscretionaryModel<C> for T
where
    T: HasTableExt,
    C: DiscretionarySameAsIndex,
    Self: TrySetNestedColumns<<Self::Table as TableExt>::Error, C::NestedHostColumns>
        + TrySetColumn<C>
        + ValidateNestedColumns<<Self::Table as TableExt>::Error, C::NestedHostColumns>,
    <Self::Table as TableExt>::Error: From<<Self as ValidateColumn<C>>::Error>,
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
        <Self as TrySetColumn<C>>::try_set_column(self, primary_key)?;
        self.try_set_nested_columns(converted_columns)
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
        link = $link:literal $(,)?
    ) => {
        #[doc = concat!(
            "Extension trait for [`", $link, "`] that allows specifying the column at \
             the method level.\n\nThis trait provides a cleaner API where the \
             relationship is chosen with a method type parameter rather than on the \
             trait itself."
        )]
        pub trait $ext: $supertrait {
            #[doc = concat!("Sets the ", $subject, " for the specified column.")]
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
                 returning `self` for fluent chaining."
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
        link = $link:literal $(,)?
    ) => {
        #[doc = concat!(
            "Extension trait for [`", $link, "`] that allows specifying the column at \
             the method level.\n\nThis trait provides a cleaner API where the \
             relationship is chosen with a method type parameter rather than on the \
             trait itself."
        )]
        pub trait $ext: $supertrait {
            #[doc = concat!(
                "Attempts to set the ", $subject, " for the specified column.\n\n\
                 # Errors\n\nReturns an error if the ", $subject, " cannot be set."
            )]
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
                 and returning `self` for fluent chaining.\n\n# Errors\n\nReturns an \
                 error if the ", $subject, " cannot be set."
            )]
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

    /// Attempt to set the value of the specified column in the mandatory
    /// same-as relationship.
    ///
    /// # Errors
    ///
    /// Returns an error if the column value cannot be set in the mandatory
    /// same-as relationship.
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

    /// Attempt to set the value of the specified column in the discretionary
    /// same-as relationship.
    ///
    /// # Errors
    ///
    /// Returns an error if the column value cannot be set in the discretionary
    /// same-as relationship.
    fn try_set_discretionary_same_as_column(
        &mut self,
        value: impl Into<C::ColumnType>,
    ) -> Result<&mut Self, Self::Error>;
}
