//! Extended `Table` trait with additional functionality.

use tuplities::prelude::{FlattenNestedTuple, IntoNestedTupleOption, NestedTupleOptionWith};

use crate::{
    NestedColumns, NonOptionalTypedNestedTuple, TableModel, TypedNestedTuple,
    columns::{NonEmptyNestedProjection, NonEmptyProjection},
};

/// Extended trait for Diesel tables.
pub trait TableExt:
    diesel::Table<AllColumns: NonEmptyProjection<Table = Self>>
    + diesel::associations::HasTable
    + Default
    + Copy
{
    /// Name of the table as a static string.
    const TABLE_NAME: &'static str;
    /// The associated Diesel model type for this table.
    type Model: TableModel<Table = Self>;
    /// The nested columns necessary to execute insert operations for this
    /// table.
    type NewRecord: NonEmptyNestedProjection<
            Table = Self,
            NestedTupleColumnType: IntoNestedTupleOption<IntoOptions = Self::NewValues>,
            Flattened: NonEmptyProjection<Table = Self>,
        >;
    /// The nested types representing a `Self::NewColumns` for this table.
    type NewValues: FlattenNestedTuple
        + NestedTupleOptionWith<
            &'static str,
            Transposed = <Self::NewRecord as TypedNestedTuple>::NestedTupleColumnType,
            SameDepth = <Self::NewRecord as NestedColumns>::NestedNames,
        >;
    /// The nested primary key columns of this table.
    type NestedPrimaryKeyColumns: NonEmptyNestedProjection<Table = Self>
        + NonOptionalTypedNestedTuple;
    /// Error type associated with this table, such as for the validation
    /// of values before insertion.
    type Error;
    /// Error type for whole-record validation of this table's new values.
    type RecordError: std::error::Error + Send + Sync + 'static;
    /// The columns of this table with an explicit `default` attribute,
    /// excluding surrogate primary keys.
    type DefaultColumns: NestedColumns;

    /// Returns the default values for the new record.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::MayGetColumn;
    /// use schema::users;
    ///
    /// let values = users::table::default_new_values();
    /// assert_eq!(MayGetColumn::<users::name>::may_get_column(&values), None);
    /// assert_eq!(MayGetColumn::<users::age>::may_get_column(&values), Some(18));
    /// assert_eq!(MayGetColumn::<users::nickname>::may_get_column(&values), Some(None),);
    /// # }
    /// ```
    #[must_use]
    fn default_new_values() -> Self::NewValues;

    /// Returns new-record values without any explicit defaults: mandatory
    /// values stay absent and nullable values are initialized to null.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::MayGetColumn;
    /// use schema::users;
    ///
    /// let values = users::table::empty_new_values();
    /// assert_eq!(MayGetColumn::<users::name>::may_get_column(&values), None);
    /// assert_eq!(MayGetColumn::<users::age>::may_get_column(&values), None);
    /// assert_eq!(MayGetColumn::<users::nickname>::may_get_column(&values), Some(None),);
    /// # }
    /// ```
    #[must_use]
    fn empty_new_values() -> Self::NewValues;
}

/// Extended trait for Diesel models associated with a table.
pub trait HasTableExt: diesel::associations::HasTable<Table: TableExt> {}

impl<T> HasTableExt for T where T: diesel::associations::HasTable<Table: TableExt> {}
