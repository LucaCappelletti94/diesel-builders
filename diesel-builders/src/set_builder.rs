//! Submodule providing the `SetBuilder` trait.

use diesel::{Table, associations::HasTable};

use crate::{
    BuildableTable, BundlableTable, CompletedTableBuilderBundle, DiscretionarySameAsIndex,
    GetColumnExt, GetColumns, HasPrimaryKey, HomogeneousColumns, HorizontalSameAsKey,
    MandatorySameAsIndex, Projection, SetColumn, SetColumns, SingletonForeignKey, TableAddition,
    TableBuilder, TrySetColumn, TrySetColumns, TypedColumn,
};

/// Trait attempting to set a specific Diesel column, which may fail.
pub trait SetMandatoryBuilder<Column: MandatorySameAsIndex<ReferencedTable: BuildableTable>> {
    /// Attempt to set the value of the specified column.
    fn set_mandatory_builder(
        &mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> &mut Self;
}

/// Trait attempting to set a specific Diesel discretionary triangular builder,
/// which may fail.
pub trait SetDiscretionaryBuilder<
    Column: crate::DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
>
{
    /// Attempt to set the value of the specified column.
    fn set_discretionary_builder(
        &mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> &mut Self;
}

/// Trait attempting to set a specific Diesel discretionary triangular model,
/// which may fail.
pub trait SetDiscretionaryModel<Column: crate::DiscretionarySameAsIndex> {
    /// Attempt to set the values associated to the provided model.
    fn set_discretionary_model(
        &mut self,
        model: &<<Column as SingletonForeignKey>::ReferencedTable as TableAddition>::Model,
    ) -> &mut Self;
}

impl<C, T> SetDiscretionaryModel<C> for T
where
    C: crate::DiscretionarySameAsIndex,
    C::ReferencedTable: BuildableTable,
    Self: SetColumns<<C as HorizontalSameAsKey>::HostColumns> + SetColumn<C>,
    <<C as SingletonForeignKey>::ReferencedTable as TableAddition>::Model:
        GetColumns<<C as HorizontalSameAsKey>::ForeignColumns>,
{
    #[inline]
    fn set_discretionary_model(
        &mut self,
        model: &<<C as SingletonForeignKey>::ReferencedTable as TableAddition>::Model,
    ) -> &mut Self {
        let primary_key = model.get_column::<<C::ReferencedTable as Table>::PrimaryKey>();
        <Self as SetColumn<C>>::set_column(self, primary_key);
        let columns = model.get_columns();
        self.set_columns(columns)
    }
}

/// Trait attempting to set a specific Diesel column, which may fail.
pub trait TrySetMandatoryBuilder<Column: MandatorySameAsIndex<ReferencedTable: BuildableTable>> {
    /// Attempt to set the value of the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the column cannot be set.
    fn try_set_mandatory_builder(
        &mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> anyhow::Result<&mut Self>;
}

/// Trait attempting to set a specific Diesel column, which may fail.
pub trait TrySetDiscretionaryBuilder<
    Column: crate::DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
>
{
    /// Attempt to set the value of the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the column cannot be set.
    fn try_set_discretionary_builder(
        &mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> anyhow::Result<&mut Self>;
}

/// Trait attempting to set a specific Diesel discretionary triangular model,
/// which may fail.
pub trait TrySetDiscretionaryModel<Column: crate::DiscretionarySameAsIndex> {
    /// Attempt to set the values associated to the provided model.
    ///
    /// # Errors
    ///
    /// Returns an error if the model cannot be set.
    fn try_set_discretionary_model(
        &mut self,
        model: &<<Column as SingletonForeignKey>::ReferencedTable as TableAddition>::Model,
    ) -> anyhow::Result<&mut Self>;
}

impl<C, T> TrySetDiscretionaryModel<C> for T
where
    C: crate::DiscretionarySameAsIndex,
    C::ReferencedTable: BuildableTable,
    Self: TrySetColumns<<C as HorizontalSameAsKey>::HostColumns> + TrySetColumn<C>,
    <<C as SingletonForeignKey>::ReferencedTable as TableAddition>::Model:
        GetColumns<<C as HorizontalSameAsKey>::ForeignColumns>,
{
    #[inline]
    fn try_set_discretionary_model(
        &mut self,
        model: &<<C as SingletonForeignKey>::ReferencedTable as TableAddition>::Model,
    ) -> anyhow::Result<&mut Self> {
        let primary_key = model.get_column::<<C::ReferencedTable as Table>::PrimaryKey>();
        <Self as TrySetColumn<C>>::try_set_column(self, primary_key)?;
        let columns = model.get_columns();
        self.try_set_columns(columns)
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
    fn set_mandatory_builder_ref<Column>(
        &mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> &mut Self
    where
        Column: MandatorySameAsIndex<ReferencedTable: BuildableTable>,
        Self: SetMandatoryBuilder<Column>,
    {
        <Self as SetMandatoryBuilder<Column>>::set_mandatory_builder(self, builder)
    }

    #[inline]
    #[must_use]
    /// Set the mandatory builder for the specified column.
    fn set_mandatory_builder<Column>(
        mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> Self
    where
        Column: MandatorySameAsIndex<ReferencedTable: BuildableTable>,
        Self: SetMandatoryBuilder<Column>,
    {
        self.set_mandatory_builder_ref::<Column>(builder);
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
    fn set_discretionary_builder_ref<Column>(
        &mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> &mut Self
    where
        Column: crate::DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
        Self: SetDiscretionaryBuilder<Column>,
    {
        <Self as SetDiscretionaryBuilder<Column>>::set_discretionary_builder(self, builder)
    }

    #[inline]
    #[must_use]
    /// Set the discretionary builder for the specified column.
    fn set_discretionary_builder<Column>(
        mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> Self
    where
        Column: crate::DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
        Self: SetDiscretionaryBuilder<Column>,
    {
        self.set_discretionary_builder_ref::<Column>(builder);
        self
    }
}

impl<T> SetDiscretionaryBuilderExt for T {}

/// Extension trait for `TrySetMandatoryBuilder` that allows specifying the
/// column at the method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
pub trait TrySetMandatoryBuilderExt: Sized {
    /// Attempt to set the mandatory builder for the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the builder cannot be set for the mandatory
    /// relationship.
    #[inline]
    fn try_set_mandatory_builder_ref<Column>(
        &mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> anyhow::Result<&mut Self>
    where
        Column: MandatorySameAsIndex<ReferencedTable: BuildableTable>,
        Self: TrySetMandatoryBuilder<Column>,
    {
        <Self as TrySetMandatoryBuilder<Column>>::try_set_mandatory_builder(self, builder)
    }

    #[inline]
    /// Attempt to set the mandatory builder for the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the builder cannot be set for the mandatory
    /// relationship.
    fn try_set_mandatory_builder<Column>(
        mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> anyhow::Result<Self>
    where
        Column: MandatorySameAsIndex<ReferencedTable: BuildableTable>,
        Self: TrySetMandatoryBuilder<Column>,
    {
        self.try_set_mandatory_builder_ref::<Column>(builder)?;
        Ok(self)
    }
}

impl<T> TrySetMandatoryBuilderExt for T {}

/// Extension trait for `TrySetDiscretionaryBuilder` that allows specifying the
/// column at the method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
pub trait TrySetDiscretionaryBuilderExt: Sized {
    /// Attempt to set the discretionary builder for the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the builder cannot be set for the discretionary
    /// relationship.
    #[inline]
    fn try_set_discretionary_builder_ref<Column>(
        &mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> anyhow::Result<&mut Self>
    where
        Column: crate::DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
        Self: TrySetDiscretionaryBuilder<Column>,
    {
        <Self as TrySetDiscretionaryBuilder<Column>>::try_set_discretionary_builder(self, builder)
    }

    #[inline]
    /// Attempt to set the discretionary builder for the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the builder cannot be set for the discretionary
    /// relationship.
    fn try_set_discretionary_builder<Column>(
        mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> anyhow::Result<Self>
    where
        Column: crate::DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
        Self: TrySetDiscretionaryBuilder<Column>,
    {
        self.try_set_discretionary_builder_ref::<Column>(builder)?;
        Ok(self)
    }
}

impl<T> TrySetDiscretionaryBuilderExt for T {}

/// Extension trait for `SetDiscretionaryModel` that allows specifying the
/// column at the method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
pub trait SetDiscretionaryModelExt: Sized {
    /// Set the discretionary model for the specified column.
    #[inline]
    fn set_discretionary_model_ref<Column>(
        &mut self,
        model: &<<Column as SingletonForeignKey>::ReferencedTable as TableAddition>::Model,
    ) -> &mut Self
    where
        Column: crate::DiscretionarySameAsIndex,
        Self: SetDiscretionaryModel<Column>,
    {
        <Self as SetDiscretionaryModel<Column>>::set_discretionary_model(self, model)
    }

    #[inline]
    #[must_use]
    /// Set the discretionary model for the specified column.
    fn set_discretionary_model<Column>(
        mut self,
        model: &<<Column as SingletonForeignKey>::ReferencedTable as TableAddition>::Model,
    ) -> Self
    where
        Column: crate::DiscretionarySameAsIndex,
        Self: SetDiscretionaryModel<Column>,
    {
        self.set_discretionary_model_ref::<Column>(model);
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
    fn try_set_discretionary_model_ref<Column>(
        &mut self,
        model: &<<Column as SingletonForeignKey>::ReferencedTable as TableAddition>::Model,
    ) -> anyhow::Result<&mut Self>
    where
        Column: crate::DiscretionarySameAsIndex,
        Self: TrySetDiscretionaryModel<Column>,
    {
        <Self as TrySetDiscretionaryModel<Column>>::try_set_discretionary_model(self, model)
    }

    #[inline]
    /// Attempt to set the discretionary model for the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the model cannot be set for the discretionary
    /// relationship.
    fn try_set_discretionary_model<Column>(
        mut self,
        model: &<<Column as SingletonForeignKey>::ReferencedTable as TableAddition>::Model,
    ) -> anyhow::Result<Self>
    where
        Column: crate::DiscretionarySameAsIndex,
        Self: TrySetDiscretionaryModel<Column>,
    {
        self.try_set_discretionary_model_ref::<Column>(model)?;
        Ok(self)
    }
}

impl<T> TrySetDiscretionaryModelExt for T {}

/// Trait to try set a column in a mandatory same-as relationship.
pub trait TrySetMandatorySameAsColumn<
    Key: MandatorySameAsIndex,
    Column: TypedColumn<Table = Key::ReferencedTable>,
>
{
    /// Attempt to set the value of the specified column in the mandatory
    /// same-as relationship.
    ///
    /// # Errors
    ///
    /// Returns an error if the column value cannot be set in the mandatory
    /// same-as relationship.
    fn try_set_mandatory_same_as_column(
        &mut self,
        value: &<Column as TypedColumn>::Type,
    ) -> anyhow::Result<&mut Self>;
}

/// Trait to try set a column in a mandatory same-as relationship.
pub trait TrySetMandatorySameAsColumns<
    Type,
    Keys: Projection<Self::Table>,
    CS: HomogeneousColumns<Type>,
>: HasTable
{
    /// Attempt to set the value of the specified columns in the mandatory
    /// same-as relationship.
    ///
    /// # Errors
    ///
    /// Returns an error if the column values cannot be set in the mandatory
    /// same-as relationship.
    fn try_set_mandatory_same_as_columns(&mut self, value: &Type) -> anyhow::Result<&mut Self>;
}

/// Trait to try set a column in a discretionary same-as relationship.
pub trait TryMaySetDiscretionarySameAsColumn<
    Key: DiscretionarySameAsIndex,
    Column: TypedColumn<Table = Key::ReferencedTable>,
>
{
    /// Attempt to set the value of the specified column in the discretionary
    /// same-as relationship.
    ///
    /// # Errors
    ///
    /// Returns an error if the column value cannot be set in the discretionary
    /// same-as relationship.
    fn try_may_set_discretionary_same_as_column(
        &mut self,
        value: &<Column as TypedColumn>::Type,
    ) -> anyhow::Result<&mut Self>;
}

/// Trait to try set a column in a discretionary same-as relationship.
#[diesel_builders_macros::impl_try_set_same_as_columns]
pub trait TryMaySetDiscretionarySameAsColumns<
    Type,
    Keys: Projection<Self::Table>,
    CS: HomogeneousColumns<Type>,
>: HasTable
{
    /// Attempt to set the value of the specified columns in the discretionary
    /// same-as relationship.
    ///
    /// # Errors
    ///
    /// Returns an error if the column values cannot be set in the discretionary
    /// same-as relationship.
    fn try_may_set_discretionary_same_as_columns(
        &mut self,
        value: &Type,
    ) -> anyhow::Result<&mut Self>;
}
