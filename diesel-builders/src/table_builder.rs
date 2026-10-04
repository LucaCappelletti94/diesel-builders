//! Submodule defining the `TableBuilder` struct for building Diesel table
//! insertables.

use std::convert::Infallible;

use diesel::{Table, associations::HasTable};
use tuplities::prelude::*;

use crate::mutation::{
    ColumnInput, MutationContext, PrepareColumn, PrepareHomogeneous, PrepareOptionalColumns,
};

mod completed_table_builder;
/// Delegation impls of the same-as side-preparation traits for `TableBuilder`.
mod mutation;
mod serde;
pub use completed_table_builder::{RecursiveBuilderInsert, RecursiveTableBuilder};

use crate::{
    AncestorOfIndex, BundlableTable, ColumnTyped, DescendantOf, DiscretionarySameAsIndex,
    ForeignPrimaryKey, MandatorySameAsIndex, MayGetColumn, MayGetNestedColumns, NestedColumns,
    SetColumn, SetDiscretionaryBuilder, SetHomogeneousNestedColumns, SetMandatoryBuilder,
    TableBuilderBundle, TableExt, TrySetColumn, TrySetDiscretionaryBuilder, TrySetMandatoryBuilder,
    TypedColumn, ValidateColumn, buildable_table::BuildableTable,
    builder_bundle::BundlableTableExt, vertical_same_as_group::VerticalSameAsGroup,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// A builder for creating insertable models for a Diesel table and its
/// ancestors.
///
/// # Type Parameters
///
/// * `T`: The table type this builder is for, must implement `BuildableTable`
///
/// # Examples
///
/// Build a record with a triangular dependency and insert it.
///
/// ```rust
/// # include!("doctest_setup.rs");
/// # use schema::*;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let mut conn = connection()?;
/// let child = children::table::builder()
///     .mandatory(sides::table::builder())
///     .discretionary(sides::table::builder())
///     .child_label("short")
///     .insert(&mut conn)?;
/// let mandatory: Side = child.mandatory(&mut conn)?;
/// assert_eq!(mandatory.get_column::<sides::parent_id>(), child.get_column::<children::id>());
/// # Ok(())
/// # }
/// ```
pub struct TableBuilder<T: BuildableTable> {
    /// The insertable models for the table and its ancestors.
    pub(crate) bundles: T::NestedAncestorBuilders,
}

impl<T: BuildableTable> TableBuilder<T> {
    /// Creates a new `TableBuilder` from the given checked bundles.
    ///
    /// # Examples
    ///
    /// Wrap a checked bundle and insert the record it stages.
    ///
    /// ```rust
    /// # include!("doctest_setup.rs");
    /// # use schema::*;
    /// # use diesel_builders::TableBuilderBundle;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut conn = connection()?;
    /// let bundle =
    ///     TableBuilderBundle::<users::table>::try_from_values(user_values("Ada", 20, Some("Ace")))
    ///         .expect("valid values must be accepted");
    /// let builder = TableBuilder::<users::table>::from_bundles((bundle,));
    /// let user = builder.insert(&mut conn)?;
    /// assert_eq!(user.get_column::<users::name>(), "Ada");
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_bundles(bundles: T::NestedAncestorBuilders) -> Self {
        Self { bundles }
    }
}

impl<T> Default for TableBuilder<T>
where
    T: BuildableTable,
    T::DefaultError: Into<Infallible>,
{
    #[inline]
    fn default() -> Self {
        <T as BuildableTable>::builder()
    }
}

/// Routes a [`TableBuilder`] to the [`TableBuilderBundle`] of one of its
/// ancestor tables.
///
/// Every accessor reaches the bundle owning the ancestor table it touches.
/// This trait names that `NestedTupleIndex` projection once so the accessor
/// impls carry a single `Self: AncestorBundle<A>` bound instead of restating
/// it.
trait AncestorBundle<A: BundlableTableExt> {
    /// Returns a shared reference to the bundle of ancestor table `A`.
    fn ancestor_bundle(&self) -> &TableBuilderBundle<A>;
}

/// Mutable counterpart of [`AncestorBundle`] used by the setting accessors.
trait AncestorBundleMut<A: BundlableTableExt>: AncestorBundle<A> {
    /// Returns a mutable reference to the bundle of ancestor table `A`.
    fn ancestor_bundle_mut(&mut self) -> &mut TableBuilderBundle<A>;
}

impl<A, T> AncestorBundle<A> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<A>,
    A: BundlableTableExt + AncestorOfIndex<T>,
    T::NestedAncestorBuilders:
        NestedTupleIndex<<A as AncestorOfIndex<T>>::Idx, Element = TableBuilderBundle<A>>,
{
    #[inline]
    fn ancestor_bundle(&self) -> &TableBuilderBundle<A> {
        self.bundles.nested_index()
    }
}

impl<A, T> AncestorBundleMut<A> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<A>,
    A: BundlableTableExt + AncestorOfIndex<T>,
    T::NestedAncestorBuilders:
        NestedTupleIndexMut<<A as AncestorOfIndex<T>>::Idx, Element = TableBuilderBundle<A>>,
{
    #[inline]
    fn ancestor_bundle_mut(&mut self) -> &mut TableBuilderBundle<A> {
        self.bundles.nested_index_mut()
    }
}

impl<T> HasTable for TableBuilder<T>
where
    T: BuildableTable,
{
    type Table = T;

    #[inline]
    fn table() -> Self::Table {
        T::default()
    }
}

impl<C, T> MayGetColumn<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn<Table: 'static>,
    C::Table: AncestorOfIndex<T> + BundlableTable,
    TableBuilderBundle<C::Table>: MayGetColumn<C>,
    Self: AncestorBundle<C::Table>,
{
    #[inline]
    fn may_get_column(&self) -> Option<C::ColumnType> {
        self.ancestor_bundle().may_get_column()
    }

    #[inline]
    fn may_get_column_ref(&self) -> Option<&C::ColumnType> {
        self.ancestor_bundle().may_get_column_ref()
    }
}

impl<C, T> ValidateColumn<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn,
    C::Table: AncestorOfIndex<T> + BundlableTable,
    TableBuilderBundle<C::Table>: ValidateColumn<C>,
{
    type Error = <TableBuilderBundle<C::Table> as ValidateColumn<C>>::Error;

    #[inline]
    fn validate_column(value: &C::ValueType) -> Result<(), Self::Error> {
        <TableBuilderBundle<C::Table> as ValidateColumn<C>>::validate_column(value)
    }
}

impl<C, T> SetColumn<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: VerticalSameAsGroup,
    Self: SetHomogeneousNestedColumns<C::ValueType, C::VerticalSameAsNestedColumns>
        + AncestorBundleMut<C::Table>,
    C::Table: AncestorOfIndex<T> + BundlableTable,
    TableBuilderBundle<C::Table>: SetColumn<C>,
{
    #[inline]
    fn set_column(&mut self, value: impl Into<C::ColumnType>) -> &mut Self {
        let value = value.into();
        // We set eventual vertically-same-as columns in nested builders first.
        self.set_homogeneous_nested_columns(&value);
        self.ancestor_bundle_mut().set_column(value);
        self
    }
}

/// A prepared column split between the ancestor-owned and the homogeneous
/// nested parts.
pub(crate) struct PreparedBuilderColumn<Own, Vertical> {
    /// The prepared column value of the ancestor table.
    own: Own,
    /// The prepared homogeneous nested column values.
    vertical: Vertical,
}

impl<C: TypedColumn, Own: ColumnInput<C>, Vertical> ColumnInput<C>
    for PreparedBuilderColumn<Own, Vertical>
{
    fn value(&self) -> &C::ColumnType {
        self.own.value()
    }

    fn into_value(self) -> C::ColumnType {
        self.own.into_value()
    }
}

impl<C, T> PrepareColumn<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: VerticalSameAsGroup,
    C::Table: AncestorOfIndex<T> + BundlableTable,
    TableBuilderBundle<C::Table>: PrepareColumn<C>,
    Self: PrepareHomogeneous<Self::Error, C::ValueType, C::VerticalSameAsNestedColumns>
        + AncestorBundleMut<C::Table>,
{
    type Prepared =
        PreparedBuilderColumn<
            <TableBuilderBundle<C::Table> as PrepareColumn<C>>::Prepared,
            <Self as PrepareHomogeneous<
                Self::Error,
                C::ValueType,
                C::VerticalSameAsNestedColumns,
            >>::Prepared,
        >;

    fn prepare_column(
        &self,
        value: C::ColumnType,
        context: &MutationContext,
    ) -> Result<Self::Prepared, (C::ColumnType, Self::Error)> {
        let own = self.ancestor_bundle().prepare_column(value, context)?;
        match self.prepare_homogeneous(own.value(), context) {
            Ok(vertical) => Ok(PreparedBuilderColumn { own, vertical }),
            Err(error) => Err((own.into_value(), error)),
        }
    }

    fn apply_column(&mut self, prepared: Self::Prepared) {
        self.apply_homogeneous(prepared.vertical);
        self.ancestor_bundle_mut().apply_column(prepared.own);
    }
}

impl<C, T> TrySetColumn<C> for TableBuilder<T>
where
    T: BuildableTable,
    C: TypedColumn,
    Self: PrepareColumn<C>,
{
    #[inline]
    fn try_set_column(
        &mut self,
        value: impl Into<C::ColumnType>,
    ) -> Result<&mut Self, Self::Error> {
        let prepared = self
            .prepare_column(value.into(), &MutationContext::default())
            .map_err(|(_, error)| error)?;
        self.apply_column(prepared);
        Ok(self)
    }
}

/// Emits the `TableBuilder<T>` triangular builder-setter impls.
///
/// The mandatory and discretionary variants share one body per fallibility:
/// read the referenced builder's nested foreign columns, propagate them into
/// this builder, and delegate the builder itself to the ancestor bundle. They
/// differ only in the key marker, the bundle trait delegated to, the method
/// name, and (for the fallible mandatory case) an extra single-column primary
/// key bound.
macro_rules! impl_builder_setters {
    (
        @fallible
        trait = $trait:ident,
        method = $method:ident,
        index = $index:ident,
        bundle_trait = $bundle_trait:ident,
        attach_method = $attach_method:ident,
        context_fn = $context_fn:ident,
        descendant = [$($descendant:tt)+] $(,)?
    ) => {
        impl<Key, T> $trait<Key> for TableBuilder<T>
        where
            T: BuildableTable + $($descendant)+,
            Key: $index,
            Key::Table: AncestorOfIndex<T> + BuildableTable,
            Key::ReferencedTable: BuildableTable,
            Self: PrepareOptionalColumns<T::Error, Key::NestedHostColumns>
                + AncestorBundleMut<Key::Table>,
            TableBuilder<Key::ReferencedTable>: MayGetNestedColumns<Key::NestedForeignColumns>,
            TableBuilderBundle<Key::Table>: $bundle_trait<Key>,
            T::Error: From<<Key::Table as TableExt>::Error>,
        {
            #[inline]
            fn $method(
                &mut self,
                builder: TableBuilder<<Key as ForeignPrimaryKey>::ReferencedTable>,
            ) -> Result<&mut Self, T::Error> {
                let columns = builder.may_get_nested_columns();
                let converted_columns = columns.nested_tuple_option_into();
                let context = MutationContext::$context_fn(
                    std::ptr::from_ref(self.ancestor_bundle()).cast::<()>(),
                    <<Key as $index>::Idx as typenum::Unsigned>::USIZE,
                );
                let prepared = <Self as PrepareOptionalColumns<T::Error, Key::NestedHostColumns>>::prepare_optional_columns(
                    self, converted_columns, &context,
                ).map_err(|(_, error)| error)?;
                <Self as PrepareOptionalColumns<T::Error, Key::NestedHostColumns>>::apply_optional_columns(self, prepared);
                self.ancestor_bundle_mut().$attach_method(builder);
                Ok(self)
            }
        }
    };
    (
        @infallible
        trait = $trait:ident,
        method = $method:ident,
        index = $index:ident,
        bundle_trait = $bundle_trait:ident,
        context_fn = $context_fn:ident $(,)?
    ) => {
        impl<Key, T> $trait<Key> for TableBuilder<T>
        where
            T: BuildableTable + DescendantOf<Key::Table>,
            Key: $index,
            Key::Table: AncestorOfIndex<T> + BuildableTable,
            Key::ReferencedTable: BuildableTable,
            Self: PrepareOptionalColumns<Infallible, Key::NestedHostColumns> + AncestorBundleMut<Key::Table>,
            TableBuilderBundle<Key::Table>: $bundle_trait<Key>,
            TableBuilder<<Key as ForeignPrimaryKey>::ReferencedTable>:
                MayGetNestedColumns<Key::NestedForeignColumns>,
        {
            #[inline]
            fn $method(
                &mut self,
                builder: TableBuilder<<Key as ForeignPrimaryKey>::ReferencedTable>,
            ) -> &mut Self {
                let columns = builder.may_get_nested_columns();
                let converted_columns = columns.nested_tuple_option_into();
                let context = MutationContext::$context_fn(
                    std::ptr::from_ref(self.ancestor_bundle()).cast::<()>(),
                    <<Key as $index>::Idx as typenum::Unsigned>::USIZE,
                );
                let prepared = match <Self as PrepareOptionalColumns<Infallible, Key::NestedHostColumns>>::prepare_optional_columns(self, converted_columns, &context) {
                    Ok(prepared) => prepared,
                    Err((_, error)) => match error {},
                };
                <Self as PrepareOptionalColumns<Infallible, Key::NestedHostColumns>>::apply_optional_columns(self, prepared);
                self.ancestor_bundle_mut().$method(builder);
                self
            }
        }
    };
}

impl_builder_setters! {
    @fallible
    trait = TrySetMandatoryBuilder,
    method = try_set_mandatory_builder,
    index = MandatorySameAsIndex,
    bundle_trait = SetMandatoryBuilder,
    attach_method = set_mandatory_builder,
    context_fn = excluding_mandatory,
    descendant = [DescendantOf<
        Key::Table,
        NestedPrimaryKeyColumns: NestedColumns<
            NestedTupleColumnType = (<<Key::Table as Table>::PrimaryKey as ColumnTyped>::ColumnType,),
        >,
    >],
}

impl_builder_setters! {
    @fallible
    trait = TrySetDiscretionaryBuilder,
    method = try_set_discretionary_builder,
    index = DiscretionarySameAsIndex,
    bundle_trait = SetDiscretionaryBuilder,
    attach_method = set_discretionary_builder,
    context_fn = excluding_discretionary,
    descendant = [DescendantOf<Key::Table>],
}

impl_builder_setters! {
    @infallible
    trait = SetMandatoryBuilder,
    method = set_mandatory_builder,
    index = MandatorySameAsIndex,
    bundle_trait = SetMandatoryBuilder,
    context_fn = excluding_mandatory,
}

impl_builder_setters! {
    @infallible
    trait = SetDiscretionaryBuilder,
    method = set_discretionary_builder,
    index = DiscretionarySameAsIndex,
    bundle_trait = SetDiscretionaryBuilder,
    context_fn = excluding_discretionary,
}
