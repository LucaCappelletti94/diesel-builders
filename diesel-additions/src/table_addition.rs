//! Extended `Table` trait with additional functionality.

/// Extended trait for Diesel tables.
pub trait TableAddition: diesel::Table<AllColumns = Self::Columns> {
    /// Columns of the table.
    type Columns: crate::Columns;
}

impl<T> TableAddition for T
where
    T: diesel::Table,
    T::AllColumns: crate::Columns,
{
    type Columns = T::AllColumns;
}

/// Extended trait for Diesel models associated with a table.
pub trait HasTableAddition: diesel::associations::HasTable<Table = Self::TableAddition> {
    /// The table associated with the model.
    type TableAddition: TableAddition;
}

impl<T> HasTableAddition for T
where
    T: diesel::associations::HasTable,
    T::Table: TableAddition,
{
    type TableAddition = T::Table;
}
