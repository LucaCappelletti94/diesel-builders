//! Submodule providing the `SetBuilder` trait.

use diesel::Table;

use crate::{
    BuildableTable, DiscretionarySameAsIndex, GetColumnExt, GetNestedColumns, HasTableExt,
    InsertableTableModel, MandatorySameAsIndex, SetColumn, SetNestedColumns, SingletonForeignKey,
    TableBuilder, TableExt, TrySetColumn, TrySetNestedColumns, Typed, TypedColumn,
    TypedNestedTuple,
};

/// Trait attempting to set a specific Diesel column, which may fail.
pub trait SetMandatoryBuilder<Key: MandatorySameAsIndex<ReferencedTable: BuildableTable>> {
    /// Attempt to set the value of the specified column.
    fn set_mandatory_builder(&mut self, builder: TableBuilder<Key::ReferencedTable>) -> &mut Self;
}

/// Trait attempting to set a specific Diesel discretionary triangular builder,
/// which may fail.
pub trait SetDiscretionaryBuilder<Key: DiscretionarySameAsIndex<ReferencedTable: BuildableTable>> {
    /// Attempt to set the value of the specified column.
    fn set_discretionary_builder(
        &mut self,
        builder: TableBuilder<Key::ReferencedTable>,
    ) -> &mut Self;
}

/// Trait attempting to set a specific Diesel discretionary triangular model,
/// which may fail.
pub trait SetDiscretionaryModel<Key: DiscretionarySameAsIndex> {
    /// Attempt to set the values associated to the provided model.
    fn set_discretionary_model(
        &mut self,
        model: &<Key::ReferencedTable as TableExt>::Model,
    ) -> &mut Self;
}

impl<C, T> SetDiscretionaryModel<C> for T
where
    C: DiscretionarySameAsIndex,
    C::NestedForeignColumns: TypedNestedTuple<
        NestedTupleType = <C::NestedHostColumns as TypedNestedTuple>::NestedTupleType,
    >,
    Self: SetNestedColumns<C::NestedHostColumns> + SetColumn<C>,
    <<C as SingletonForeignKey>::ReferencedTable as TableExt>::Model:
        GetNestedColumns<C::NestedForeignColumns>,
{
    #[inline]
    fn set_discretionary_model(
        &mut self,
        model: &<<C as SingletonForeignKey>::ReferencedTable as TableExt>::Model,
    ) -> &mut Self {
        let primary_key = model.get_column::<<C::ReferencedTable as Table>::PrimaryKey>();
        <Self as SetColumn<C>>::set_column(self, primary_key);
        let columns = model.get_nested_columns();
        self.set_nested_columns(columns)
    }
}

/// Trait attempting to set a specific Diesel column, which may fail.
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
        builder: TableBuilder<<Key as SingletonForeignKey>::ReferencedTable>,
    ) -> Result<
        &mut Self,
        <<Self::Table as TableExt>::InsertableModel as InsertableTableModel>::Error,
    >;
}

/// Trait attempting to set a specific Diesel column, which may fail.
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
    ) -> Result<
        &mut Self,
        <<Self::Table as TableExt>::InsertableModel as InsertableTableModel>::Error,
    >;
}

/// Trait attempting to set a specific Diesel discretionary triangular model,
/// which may fail.
pub trait TrySetDiscretionaryModel<Key: DiscretionarySameAsIndex>: HasTableExt {
    /// Attempt to set the values associated to the provided model.
    ///
    /// # Errors
    ///
    /// Returns an error if the model cannot be set.
    fn try_set_discretionary_model(
        &mut self,
        model: &<Key::ReferencedTable as TableExt>::Model,
    ) -> Result<
        &mut Self,
        <<Self::Table as TableExt>::InsertableModel as InsertableTableModel>::Error,
    >;
}

impl<C, T> TrySetDiscretionaryModel<C> for T
where
    T: HasTableExt,
    C: DiscretionarySameAsIndex,
    C::NestedForeignColumns: TypedNestedTuple<
        NestedTupleType = <C::NestedHostColumns as TypedNestedTuple>::NestedTupleType,
    >,
    Self: TrySetNestedColumns<
            <<Self::Table as TableExt>::InsertableModel as InsertableTableModel>::Error,
            C::NestedHostColumns,
        > + TrySetColumn<C>,
    <<Self::Table as TableExt>::InsertableModel as InsertableTableModel>::Error:
        From<<Self as TrySetColumn<C>>::Error>,
    <<C as SingletonForeignKey>::ReferencedTable as TableExt>::Model:
        GetNestedColumns<C::NestedForeignColumns>,
{
    #[inline]
    fn try_set_discretionary_model(
        &mut self,
        model: &<<C as SingletonForeignKey>::ReferencedTable as TableExt>::Model,
    ) -> Result<
        &mut Self,
        <<Self::Table as TableExt>::InsertableModel as InsertableTableModel>::Error,
    > {
        let primary_key: C::Type = model.get_column::<<C::ReferencedTable as Table>::PrimaryKey>();
        <Self as TrySetColumn<C>>::try_set_column(self, primary_key)?;
        let columns = model.get_nested_columns();
        self.try_set_nested_columns(columns)
    }
}

/// Extension trait for `SetMandatoryBuilder` that allows specifying the column
/// at the method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
pub trait SetMandatoryBuilderExt: Sized {
    /// Set the mandatory builder for the specified column.
    #[inline]
    fn set_mandatory_builder_ref<Key>(
        &mut self,
        builder: TableBuilder<Key::ReferencedTable>,
    ) -> &mut Self
    where
        Key: MandatorySameAsIndex<ReferencedTable: BuildableTable>,
        Self: SetMandatoryBuilder<Key>,
    {
        <Self as SetMandatoryBuilder<Key>>::set_mandatory_builder(self, builder)
    }

    #[inline]
    #[must_use]
    /// Set the mandatory builder for the specified column.
    fn set_mandatory_builder<Key>(mut self, builder: TableBuilder<Key::ReferencedTable>) -> Self
    where
        Key: MandatorySameAsIndex<ReferencedTable: BuildableTable>,
        Self: SetMandatoryBuilder<Key>,
    {
        self.set_mandatory_builder_ref::<Key>(builder);
        self
    }
}

impl<T> SetMandatoryBuilderExt for T {}

/// Extension trait for `SetDiscretionaryBuilder` that allows specifying the
/// column at the method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
pub trait SetDiscretionaryBuilderExt: Sized {
    /// Set the discretionary builder for the specified column.
    #[inline]
    fn set_discretionary_builder_ref<Key>(
        &mut self,
        builder: TableBuilder<Key::ReferencedTable>,
    ) -> &mut Self
    where
        Key: DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
        Self: SetDiscretionaryBuilder<Key>,
    {
        <Self as SetDiscretionaryBuilder<Key>>::set_discretionary_builder(self, builder)
    }

    #[inline]
    #[must_use]
    /// Set the discretionary builder for the specified column.
    fn set_discretionary_builder<Key>(mut self, builder: TableBuilder<Key::ReferencedTable>) -> Self
    where
        Key: DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
        Self: SetDiscretionaryBuilder<Key>,
    {
        self.set_discretionary_builder_ref::<Key>(builder);
        self
    }
}

impl<T> SetDiscretionaryBuilderExt for T {}

/// Extension trait for `TrySetMandatoryBuilder` that allows specifying the
/// column at the method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
pub trait TrySetMandatoryBuilderExt: HasTableExt {
    /// Attempt to set the mandatory builder for the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the builder cannot be set for the mandatory
    /// relationship.
    #[inline]
    fn try_set_mandatory_builder_ref<Key>(
        &mut self,
        builder: TableBuilder<Key::ReferencedTable>,
    ) -> Result<
        &mut Self,
        <<Self::Table as TableExt>::InsertableModel as InsertableTableModel>::Error,
    >
    where
        Key: MandatorySameAsIndex<ReferencedTable: BuildableTable>,
        Self: TrySetMandatoryBuilder<Key>,
    {
        <Self as TrySetMandatoryBuilder<Key>>::try_set_mandatory_builder(self, builder)
    }

    #[inline]
    /// Attempt to set the mandatory builder for the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the builder cannot be set for the mandatory
    /// relationship.
    fn try_set_mandatory_builder<Key>(
        mut self,
        builder: TableBuilder<Key::ReferencedTable>,
    ) -> Result<Self, <<Self::Table as TableExt>::InsertableModel as InsertableTableModel>::Error>
    where
        Key: MandatorySameAsIndex<ReferencedTable: BuildableTable>,
        Self: TrySetMandatoryBuilder<Key> + Sized,
    {
        self.try_set_mandatory_builder_ref::<Key>(builder)?;
        Ok(self)
    }
}

impl<T: HasTableExt> TrySetMandatoryBuilderExt for T {}

/// Extension trait for `TrySetDiscretionaryBuilder` that allows specifying the
/// column at the method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
pub trait TrySetDiscretionaryBuilderExt: HasTableExt {
    /// Attempt to set the discretionary builder for the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the builder cannot be set for the discretionary
    /// relationship.
    #[inline]
    fn try_set_discretionary_builder_ref<Key>(
        &mut self,
        builder: TableBuilder<Key::ReferencedTable>,
    ) -> Result<
        &mut Self,
        <<Self::Table as TableExt>::InsertableModel as InsertableTableModel>::Error,
    >
    where
        Key: DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
        Self: TrySetDiscretionaryBuilder<Key>,
    {
        <Self as TrySetDiscretionaryBuilder<Key>>::try_set_discretionary_builder(self, builder)
    }

    #[inline]
    /// Attempt to set the discretionary builder for the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the builder cannot be set for the discretionary
    /// relationship.
    fn try_set_discretionary_builder<Key>(
        mut self,
        builder: TableBuilder<Key::ReferencedTable>,
    ) -> Result<Self, <<Self::Table as TableExt>::InsertableModel as InsertableTableModel>::Error>
    where
        Key: DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
        Self: TrySetDiscretionaryBuilder<Key> + Sized,
    {
        self.try_set_discretionary_builder_ref::<Key>(builder)?;
        Ok(self)
    }
}

impl<T: HasTableExt> TrySetDiscretionaryBuilderExt for T {}

/// Extension trait for `SetDiscretionaryModel` that allows specifying the
/// column at the method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
pub trait SetDiscretionaryModelExt: Sized {
    /// Set the discretionary model for the specified column.
    #[inline]
    fn set_discretionary_model_ref<Key>(
        &mut self,
        model: &<Key::ReferencedTable as TableExt>::Model,
    ) -> &mut Self
    where
        Key: DiscretionarySameAsIndex,
        Self: SetDiscretionaryModel<Key>,
    {
        <Self as SetDiscretionaryModel<Key>>::set_discretionary_model(self, model)
    }

    #[inline]
    #[must_use]
    /// Set the discretionary model for the specified column.
    fn set_discretionary_model<Key>(
        mut self,
        model: &<Key::ReferencedTable as TableExt>::Model,
    ) -> Self
    where
        Key: DiscretionarySameAsIndex,
        Self: SetDiscretionaryModel<Key>,
    {
        self.set_discretionary_model_ref::<Key>(model);
        self
    }
}

impl<T> SetDiscretionaryModelExt for T {}

/// Extension trait for `TrySetDiscretionaryModel` that allows specifying the
/// column at the method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
pub trait TrySetDiscretionaryModelExt: Sized {
    /// Attempt to set the discretionary model for the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the model cannot be set for the discretionary
    /// relationship.
    #[inline]
    fn try_set_discretionary_model_ref<Key>(
        &mut self,
        model: &<Key::ReferencedTable as TableExt>::Model,
    ) -> Result<
        &mut Self,
        <<Self::Table as TableExt>::InsertableModel as InsertableTableModel>::Error,
    >
    where
        Key: DiscretionarySameAsIndex,
        Self: TrySetDiscretionaryModel<Key>,
    {
        <Self as TrySetDiscretionaryModel<Key>>::try_set_discretionary_model(self, model)
    }

    #[inline]
    /// Attempt to set the discretionary model for the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the model cannot be set for the discretionary
    /// relationship.
    fn try_set_discretionary_model<Key>(
        mut self,
        model: &<Key::ReferencedTable as TableExt>::Model,
    ) -> Result<Self, <<Self::Table as TableExt>::InsertableModel as InsertableTableModel>::Error>
    where
        Key: DiscretionarySameAsIndex,
        Self: TrySetDiscretionaryModel<Key>,
    {
        self.try_set_discretionary_model_ref::<Key>(model)?;
        Ok(self)
    }
}

impl<T> TrySetDiscretionaryModelExt for T {}

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
        value: <C as Typed>::Type,
    ) -> Result<&mut Self, Self::Error>;
}

/// Trait to try set a column in a discretionary same-as relationship.
pub trait TryMaySetDiscretionarySameAsColumn<
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
    fn try_may_set_discretionary_same_as_column(
        &mut self,
        value: <C as Typed>::Type,
    ) -> Result<&mut Self, Self::Error>;
}
