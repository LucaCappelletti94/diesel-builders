//! Submodule defining the `TableBuilder` struct for building Diesel table
//! insertables.

use diesel::{Table, associations::HasTable};
use tuplities::prelude::*;

mod completed_table_builder;
mod serde;
pub use completed_table_builder::{RecursiveBuilderInsert, RecursiveTableBuilder};

use crate::{
    AncestorOfIndex, BundlableTable, DescendantOf, DiscretionarySameAsIndex, ForeignPrimaryKey,
    MandatorySameAsIndex, MayGetColumn, MayGetNestedColumns, MaySetColumns,
    MayValidateNestedColumns, NestedColumns, SetColumn, SetDiscretionaryBuilder,
    SetMandatoryBuilder, TableBuilderBundle, TableExt, TryMaySetNestedColumns, TrySetColumn,
    TrySetDiscretionaryBuilder, TrySetMandatoryBuilder, Typed, TypedColumn, TypedNestedTuple,
    ValidateColumn, buildable_table::BuildableTable, columns::NonEmptyNestedProjection,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// A builder for creating insertable models for a Diesel table and its
/// ancestors.
pub struct TableBuilder<T: BuildableTable> {
    /// The insertable models for the table and its ancestors.
    bundles: T::NestedAncestorBuilders,
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
    C: TypedColumn,
    C::Table: AncestorOfIndex<T> + BundlableTable,
    TableBuilderBundle<C::Table>: MayGetColumn<C>,
    T::NestedAncestorBuilders: NestedTupleIndex<
            <C::Table as AncestorOfIndex<T>>::Idx,
            Element = TableBuilderBundle<C::Table>,
        >,
{
    #[inline]
    fn may_get_column(&self) -> Option<<C as Typed>::Type> {
        self.bundles.nested_index().may_get_column()
    }

    #[inline]
    fn may_get_column_ref<'a>(&'a self) -> Option<&'a <C as Typed>::Type>
    where
        C::Table: 'a,
    {
        self.bundles.nested_index().may_get_column_ref()
    }
}

impl<C, T> ValidateColumn<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn,
    C::Table: AncestorOfIndex<T> + BundlableTable,
    TableBuilderBundle<C::Table>: ValidateColumn<C>,
    T::NestedAncestorBuilders: NestedTupleIndex<
            <C::Table as AncestorOfIndex<T>>::Idx,
            Element = TableBuilderBundle<C::Table>,
        >,
{
    type Error = <TableBuilderBundle<C::Table> as ValidateColumn<C>>::Error;

    #[inline]
    fn validate_column_in_context(&self, value: &<C as Typed>::Type) -> Result<(), Self::Error> {
        self.bundles
            .nested_index()
            .validate_column_in_context(value)
    }
}

impl<C, T> SetColumn<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn,
    C::Table: AncestorOfIndex<T> + BundlableTable,
    TableBuilderBundle<C::Table>: SetColumn<C>,
    T::NestedAncestorBuilders: NestedTupleIndexMut<
            <C::Table as AncestorOfIndex<T>>::Idx,
            Element = TableBuilderBundle<C::Table>,
        >,
{
    #[inline]
    fn set_column(&mut self, value: impl Into<<C as Typed>::Type>) -> &mut Self {
        self.bundles.nested_index_mut().set_column(value);
        self
    }
}

impl<C, T> TrySetColumn<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn,
    C::Table: AncestorOfIndex<T> + BundlableTable,
    TableBuilderBundle<C::Table>: TrySetColumn<C>,
    T::NestedAncestorBuilders: NestedTupleIndexMut<
            <C::Table as AncestorOfIndex<T>>::Idx,
            Element = TableBuilderBundle<C::Table>,
        >,
{
    #[inline]
    fn try_set_column(&mut self, value: <C as Typed>::Type) -> Result<&mut Self, Self::Error> {
        self.bundles.nested_index_mut().try_set_column(value)?;
        Ok(self)
    }
}

impl<Key, T> TrySetMandatoryBuilder<Key> for TableBuilder<T>
where
    T: BuildableTable
        + DescendantOf<
            Key::Table,
            NestedPrimaryKeyColumns: NestedColumns<
                NestedTupleType = (<<Key::Table as Table>::PrimaryKey as Typed>::Type,),
            >,
        >,
    Key: MandatorySameAsIndex,
    Key::NestedForeignColumns: TypedNestedTuple<
        NestedTupleType = <Key::NestedHostColumns as TypedNestedTuple>::NestedTupleType,
    >,
    Key::Table: AncestorOfIndex<T> + BuildableTable,
    Key::ReferencedTable: BuildableTable,
    Self: TryMaySetNestedColumns<T::Error, Key::NestedHostColumns>,
    TableBuilder<Key::ReferencedTable>: MayGetNestedColumns<Key::NestedForeignColumns>,
    TableBuilderBundle<Key::Table>: TrySetMandatoryBuilder<Key, Table = Key::Table>,
    T::NestedAncestorBuilders: NestedTupleIndexMut<
            <Key::Table as AncestorOfIndex<T>>::Idx,
            Element = TableBuilderBundle<Key::Table>,
        >,
    T::Error: From<<Key::Table as TableExt>::Error>,
{
    #[inline]
    fn try_set_mandatory_builder(
        &mut self,
        builder: TableBuilder<<Key as ForeignPrimaryKey>::ReferencedTable>,
    ) -> Result<&mut Self, T::Error> {
        let columns = builder.may_get_nested_columns();
        MayValidateNestedColumns::<T::Error, Key::NestedHostColumns>::may_validate_nested_columns(
            self, &columns,
        )?;
        self.bundles
            .nested_index_mut()
            .try_set_mandatory_builder(builder)?;
        self.try_may_set_nested_columns(columns)?;
        Ok(self)
    }
}

impl<C, T> SetMandatoryBuilder<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: MandatorySameAsIndex,
    C::NestedForeignColumns: TypedNestedTuple<
        NestedTupleType = <C::NestedHostColumns as TypedNestedTuple>::NestedTupleType,
    >,
    C::Table: AncestorOfIndex<T> + BuildableTable,
    C::ReferencedTable: BuildableTable,
    Self: MaySetColumns<C::NestedHostColumns>,
    TableBuilderBundle<C::Table>: SetMandatoryBuilder<C>,
    TableBuilder<<C as ForeignPrimaryKey>::ReferencedTable>:
        MayGetNestedColumns<C::NestedForeignColumns>,
    T::NestedAncestorBuilders: NestedTupleIndexMut<
            <C::Table as AncestorOfIndex<T>>::Idx,
            Element = TableBuilderBundle<C::Table>,
        >,
{
    #[inline]
    fn set_mandatory_builder(
        &mut self,
        builder: TableBuilder<<C as ForeignPrimaryKey>::ReferencedTable>,
    ) -> &mut Self {
        let columns = builder.may_get_nested_columns();
        self.may_set_nested_columns(columns);
        self.bundles
            .nested_index_mut()
            .set_mandatory_builder(builder);
        self
    }
}

impl<Key, T> TrySetDiscretionaryBuilder<Key> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<Key::Table>,
    Key: DiscretionarySameAsIndex,
    Key::NestedForeignColumns: NonEmptyNestedProjection<
        NestedTupleType = <Key::NestedHostColumns as TypedNestedTuple>::NestedTupleType,
    >,
    Key::Table: AncestorOfIndex<T> + BuildableTable,
    Key::ReferencedTable: BuildableTable,
    Self: TryMaySetNestedColumns<T::Error, Key::NestedHostColumns>,
    TableBuilder<Key::ReferencedTable>: MayGetNestedColumns<Key::NestedForeignColumns>,
    TableBuilderBundle<Key::Table>: TrySetDiscretionaryBuilder<Key, Table = Key::Table>,
    T::NestedAncestorBuilders: NestedTupleIndexMut<
            <Key::Table as AncestorOfIndex<T>>::Idx,
            Element = TableBuilderBundle<Key::Table>,
        >,
    T::Error: From<<Key::Table as TableExt>::Error>,
{
    #[inline]
    fn try_set_discretionary_builder(
        &mut self,
        builder: TableBuilder<<Key as ForeignPrimaryKey>::ReferencedTable>,
    ) -> Result<&mut Self, T::Error> {
        let columns = builder.may_get_nested_columns();
        MayValidateNestedColumns::<T::Error, Key::NestedHostColumns>::may_validate_nested_columns(
            self, &columns,
        )?;
        self.bundles
            .nested_index_mut()
            .try_set_discretionary_builder(builder)?;
        self.try_may_set_nested_columns(columns)?;
        Ok(self)
    }
}

impl<C, T> SetDiscretionaryBuilder<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: DiscretionarySameAsIndex,
    C::NestedForeignColumns: NonEmptyNestedProjection<
        NestedTupleType = <C::NestedHostColumns as TypedNestedTuple>::NestedTupleType,
    >,
    C::Table: AncestorOfIndex<T> + BuildableTable,
    C::ReferencedTable: BuildableTable,
    Self: MaySetColumns<C::NestedHostColumns>,
    TableBuilder<C::ReferencedTable>: MayGetNestedColumns<C::NestedForeignColumns>,
    TableBuilderBundle<C::Table>: SetDiscretionaryBuilder<C>,
    T::NestedAncestorBuilders: NestedTupleIndexMut<
            <C::Table as AncestorOfIndex<T>>::Idx,
            Element = TableBuilderBundle<C::Table>,
        >,
{
    #[inline]
    fn set_discretionary_builder(
        &mut self,
        builder: TableBuilder<<C as ForeignPrimaryKey>::ReferencedTable>,
    ) -> &mut Self {
        let columns = builder.may_get_nested_columns();
        self.may_set_nested_columns(columns);
        self.bundles
            .nested_index_mut()
            .set_discretionary_builder(builder);
        self
    }
}
