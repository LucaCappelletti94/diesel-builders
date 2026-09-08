//! Submodule defining the `TableBuilder` struct for building Diesel table
//! insertables.

use diesel::{Table, associations::HasTable};
use tuplities::prelude::*;

mod completed_table_builder;
mod serde;
pub use completed_table_builder::{RecursiveBuilderInsert, RecursiveTableBuilder};

use crate::{
    AncestorOfIndex, BundlableTable, ColumnTyped, DescendantOf, DiscretionarySameAsIndex,
    ForeignPrimaryKey, MandatorySameAsIndex, MayGetColumn, MayGetNestedColumns, MaySetColumns,
    MayValidateNestedColumns, NestedColumns, SetColumn, SetDiscretionaryBuilder,
    SetHomogeneousNestedColumns, SetMandatoryBuilder, TableBuilderBundle, TableExt,
    TryMaySetNestedColumns, TrySetColumn, TrySetDiscretionaryBuilder,
    TrySetHomogeneousNestedColumns, TrySetMandatoryBuilder, TypedColumn, ValidateColumn,
    buildable_table::BuildableTable, builder_bundle::BundlableTableExt,
    vertical_same_as_group::VerticalSameAsGroup,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// A builder for creating insertable models for a Diesel table and its
/// ancestors.
///
/// This struct provides a fluent API for building complex database records
/// that may have inheritance relationships or triangular dependencies. It
/// tracks the state of all required fields and ensures proper insertion order.
///
/// # Type Parameters
///
/// * `T`: The table type this builder is for, must implement `BuildableTable`
pub struct TableBuilder<T: BuildableTable> {
    /// The insertable models for the table and its ancestors.
    pub(crate) bundles: T::NestedAncestorBuilders,
}

impl<T: BuildableTable> TableBuilder<T> {
    /// Creates a new `TableBuilder` from the given bundles.
    pub fn from_bundles(bundles: T::NestedAncestorBuilders) -> Self {
        Self { bundles }
    }

    /// Consumes the builder and returns the nested ancestor bundles.
    pub fn into_bundles(self) -> T::NestedAncestorBuilders {
        self.bundles
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
    Self: AncestorBundle<C::Table>,
{
    type Error = <TableBuilderBundle<C::Table> as ValidateColumn<C>>::Error;

    #[inline]
    fn validate_column_in_context(&self, value: &C::ValueType) -> Result<(), Self::Error> {
        self.ancestor_bundle().validate_column_in_context(value)
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

impl<C, T> TrySetColumn<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: VerticalSameAsGroup,
    Self: TrySetHomogeneousNestedColumns<C::ValueType, Self::Error, C::VerticalSameAsNestedColumns>
        + AncestorBundleMut<C::Table>,
    C::Table: AncestorOfIndex<T> + BundlableTable,
    TableBuilderBundle<C::Table>: TrySetColumn<C>,
{
    #[inline]
    fn try_set_column(
        &mut self,
        value: impl Into<C::ColumnType>,
    ) -> Result<&mut Self, Self::Error> {
        let value = value.into();
        // We try to set eventual vertically-same-as columns in nested builders
        // first.
        self.try_set_homogeneous_nested_columns(&value)?;
        self.ancestor_bundle_mut().try_set_column(value)?;
        Ok(self)
    }
}

impl<Key, T> TrySetMandatoryBuilder<Key> for TableBuilder<T>
where
    T: BuildableTable
        + DescendantOf<
            Key::Table,
            NestedPrimaryKeyColumns: NestedColumns<
                NestedTupleColumnType = (
                    <<Key::Table as Table>::PrimaryKey as ColumnTyped>::ColumnType,
                ),
            >,
        >,
    Key: MandatorySameAsIndex,
    Key::Table: AncestorOfIndex<T> + BuildableTable,
    Key::ReferencedTable: BuildableTable,
    Self: TryMaySetNestedColumns<T::Error, Key::NestedHostColumns>
        + MayValidateNestedColumns<T::Error, Key::NestedHostColumns>
        + AncestorBundleMut<Key::Table>,
    TableBuilder<Key::ReferencedTable>: MayGetNestedColumns<Key::NestedForeignColumns>,
    TableBuilderBundle<Key::Table>: TrySetMandatoryBuilder<Key, Table = Key::Table>,
    T::Error: From<<Key::Table as TableExt>::Error>,
{
    #[inline]
    fn try_set_mandatory_builder(
        &mut self,
        builder: TableBuilder<<Key as ForeignPrimaryKey>::ReferencedTable>,
    ) -> Result<&mut Self, T::Error> {
        let columns = builder.may_get_nested_columns();
        let converted_columns = columns.nested_tuple_option_into();
        self.may_validate_nested_columns(&converted_columns)?;
        self.ancestor_bundle_mut().try_set_mandatory_builder(builder)?;
        self.try_may_set_nested_columns(converted_columns)?;
        Ok(self)
    }
}

impl<C, T> SetMandatoryBuilder<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: MandatorySameAsIndex,
    C::Table: AncestorOfIndex<T> + BuildableTable,
    C::ReferencedTable: BuildableTable,
    Self: MaySetColumns<C::NestedHostColumns> + AncestorBundleMut<C::Table>,
    TableBuilderBundle<C::Table>: SetMandatoryBuilder<C>,
    TableBuilder<<C as ForeignPrimaryKey>::ReferencedTable>:
        MayGetNestedColumns<C::NestedForeignColumns>,
{
    #[inline]
    fn set_mandatory_builder(
        &mut self,
        builder: TableBuilder<<C as ForeignPrimaryKey>::ReferencedTable>,
    ) -> &mut Self {
        let columns = builder.may_get_nested_columns();
        let converted_columns = columns.nested_tuple_option_into();
        self.may_set_nested_columns(converted_columns);
        self.ancestor_bundle_mut().set_mandatory_builder(builder);
        self
    }
}

impl<Key, T> TrySetDiscretionaryBuilder<Key> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<Key::Table>,
    Key: DiscretionarySameAsIndex,
    Key::Table: AncestorOfIndex<T> + BuildableTable,
    Key::ReferencedTable: BuildableTable,
    Self: TryMaySetNestedColumns<T::Error, Key::NestedHostColumns>
        + MayValidateNestedColumns<T::Error, Key::NestedHostColumns>
        + AncestorBundleMut<Key::Table>,
    TableBuilder<Key::ReferencedTable>: MayGetNestedColumns<Key::NestedForeignColumns>,
    TableBuilderBundle<Key::Table>: TrySetDiscretionaryBuilder<Key, Table = Key::Table>,
    T::Error: From<<Key::Table as TableExt>::Error>,
{
    #[inline]
    fn try_set_discretionary_builder(
        &mut self,
        builder: TableBuilder<<Key as ForeignPrimaryKey>::ReferencedTable>,
    ) -> Result<&mut Self, T::Error> {
        let columns = builder.may_get_nested_columns();
        let converted_columns = columns.nested_tuple_option_into();
        self.may_validate_nested_columns(&converted_columns)?;
        self.ancestor_bundle_mut().try_set_discretionary_builder(builder)?;
        self.try_may_set_nested_columns(converted_columns)?;
        Ok(self)
    }
}

impl<C, T> SetDiscretionaryBuilder<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: DiscretionarySameAsIndex,
    C::Table: AncestorOfIndex<T> + BuildableTable,
    C::ReferencedTable: BuildableTable,
    Self: MaySetColumns<C::NestedHostColumns> + AncestorBundleMut<C::Table>,
    TableBuilder<C::ReferencedTable>: MayGetNestedColumns<C::NestedForeignColumns>,
    TableBuilderBundle<C::Table>: SetDiscretionaryBuilder<C>,
{
    #[inline]
    fn set_discretionary_builder(
        &mut self,
        builder: TableBuilder<<C as ForeignPrimaryKey>::ReferencedTable>,
    ) -> &mut Self {
        let columns = builder.may_get_nested_columns();
        let converted_columns = columns.nested_tuple_option_into();
        self.may_set_nested_columns(converted_columns);
        self.ancestor_bundle_mut().set_discretionary_builder(builder);
        self
    }
}
