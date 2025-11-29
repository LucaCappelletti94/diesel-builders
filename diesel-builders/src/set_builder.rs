//! Submodule providing the `SetBuilder` trait.

use diesel_additions::SingletonForeignKey;
use diesel_relations::MandatorySameAsIndex;

use crate::{BuildableTable, TableBuilder};

/// Trait attempting to set a specific Diesel column, which may fail.
pub trait SetMandatoryBuilder<Column: MandatorySameAsIndex<ReferencedTable: BuildableTable>> {
    /// Attempt to set the value of the specified column.
    fn set_mandatory_builder(
        &mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> &mut Self;
}

/// Trait attempting to set a specific Diesel column, which may fail.
pub trait SetDiscretionaryBuilder<
    Column: diesel_relations::DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
>
{
    /// Attempt to set the value of the specified column.
    fn set_discretionary_builder(
        &mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> &mut Self;
}

/// Trait attempting to set a specific Diesel column, which may fail.
pub trait TrySetMandatoryBuilder<Column: MandatorySameAsIndex<ReferencedTable: BuildableTable>> {
    /// Attempt to set the value of the specified column.
    fn try_set_mandatory_builder(
        &mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> anyhow::Result<&mut Self>;
}

/// Trait attempting to set a specific Diesel column, which may fail.
pub trait TrySetDiscretionaryBuilder<
    Column: diesel_relations::DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
>
{
    /// Attempt to set the value of the specified column.
    fn try_set_discretionary_builder(
        &mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> anyhow::Result<&mut Self>;
}

/// Extension trait for `SetMandatoryBuilder` that allows specifying the column
/// at the method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
///
/// # Example
///
/// ```ignore
/// // Instead of:
/// <TableBuilder<TableB> as SetMandatoryBuilder<table_b::a_id>>::set_mandatory_builder(&mut builder, a_builder)
///
/// // You can write:
/// builder.set_mandatory_builder::<table_b::a_id>(a_builder)
/// ```
pub trait SetMandatoryBuilderExt {
    /// Set the mandatory builder for the specified column.
    fn set_mandatory_builder<Column>(
        &mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> &mut Self
    where
        Column: MandatorySameAsIndex<ReferencedTable: BuildableTable>,
        Self: SetMandatoryBuilder<Column>;
}

impl<T> SetMandatoryBuilderExt for T {
    fn set_mandatory_builder<Column>(
        &mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> &mut Self
    where
        Column: MandatorySameAsIndex<ReferencedTable: BuildableTable>,
        Self: SetMandatoryBuilder<Column>,
    {
        <Self as SetMandatoryBuilder<Column>>::set_mandatory_builder(self, builder)
    }
}

/// Extension trait for `SetDiscretionaryBuilder` that allows specifying the
/// column at the method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
///
/// # Example
///
/// ```ignore
/// // Instead of:
/// <TableBuilder<TableB> as SetDiscretionaryBuilder<table_b::a_id>>::set_discretionary_builder(&mut builder, a_builder)
///
/// // You can write:
/// builder.set_discretionary_builder::<table_b::a_id>(a_builder)
/// ```
pub trait SetDiscretionaryBuilderExt {
    /// Set the discretionary builder for the specified column.
    fn set_discretionary_builder<Column>(
        &mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> &mut Self
    where
        Column: diesel_relations::DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
        Self: SetDiscretionaryBuilder<Column>;
}

impl<T> SetDiscretionaryBuilderExt for T {
    fn set_discretionary_builder<Column>(
        &mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> &mut Self
    where
        Column: diesel_relations::DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
        Self: SetDiscretionaryBuilder<Column>,
    {
        <Self as SetDiscretionaryBuilder<Column>>::set_discretionary_builder(self, builder)
    }
}

/// Extension trait for `TrySetMandatoryBuilder` that allows specifying the
/// column at the method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
///
/// # Example
///
/// ```ignore
/// // Instead of:
/// <TableBuilder<TableB> as TrySetMandatoryBuilder<table_b::a_id>>::try_set_mandatory_builder(&mut builder, a_builder)?
///
/// // You can write:
/// builder.try_set_mandatory_builder::<table_b::a_id>(a_builder)?
/// ```
pub trait TrySetMandatoryBuilderExt {
    /// Attempt to set the mandatory builder for the specified column.
    fn try_set_mandatory_builder<Column>(
        &mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> anyhow::Result<&mut Self>
    where
        Column: MandatorySameAsIndex<ReferencedTable: BuildableTable>,
        Self: TrySetMandatoryBuilder<Column>;
}

impl<T> TrySetMandatoryBuilderExt for T {
    fn try_set_mandatory_builder<Column>(
        &mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> anyhow::Result<&mut Self>
    where
        Column: MandatorySameAsIndex<ReferencedTable: BuildableTable>,
        Self: TrySetMandatoryBuilder<Column>,
    {
        <Self as TrySetMandatoryBuilder<Column>>::try_set_mandatory_builder(self, builder)
    }
}

/// Extension trait for `TrySetDiscretionaryBuilder` that allows specifying the
/// column at the method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
///
/// # Example
///
/// ```ignore
/// // Instead of:
/// <TableBuilder<TableB> as TrySetDiscretionaryBuilder<table_b::a_id>>::try_set_discretionary_builder(&mut builder, a_builder)?
///
/// // You can write:
/// builder.try_set_discretionary_builder::<table_b::a_id>(a_builder)?
/// ```
pub trait TrySetDiscretionaryBuilderExt {
    /// Attempt to set the discretionary builder for the specified column.
    fn try_set_discretionary_builder<Column>(
        &mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> anyhow::Result<&mut Self>
    where
        Column: diesel_relations::DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
        Self: TrySetDiscretionaryBuilder<Column>;
}

impl<T> TrySetDiscretionaryBuilderExt for T {
    fn try_set_discretionary_builder<Column>(
        &mut self,
        builder: TableBuilder<<Column as SingletonForeignKey>::ReferencedTable>,
    ) -> anyhow::Result<&mut Self>
    where
        Column: diesel_relations::DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
        Self: TrySetDiscretionaryBuilder<Column>,
    {
        <Self as TrySetDiscretionaryBuilder<Column>>::try_set_discretionary_builder(self, builder)
    }
}
