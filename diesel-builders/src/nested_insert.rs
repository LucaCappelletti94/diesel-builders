//! Submodule defining the `Insert` trait, which executes the insertion of a
//! builder into the database, following the dependencies between tables.

use crate::{BuilderResult, DescendantWithSelf, HasTableExt, NestedTables, TableExt};

/// Trait defining the insertion of a builder into the database.
pub trait Insert<Conn>: HasTableExt<Table: DescendantWithSelf> {
    /// The validation errors that atomic insertion of this builder can still
    /// produce.
    type ValidationError;

    /// Atomically inserts the builder, its ancestors, and its associated
    /// records.
    ///
    /// # Errors
    ///
    /// Returns an error if the insertion fails, if any remaining validation
    /// fails, or if any database constraints are violated.
    ///
    /// # Examples
    ///
    /// Insert the record and its associated records, and roll back when a
    /// database constraint is violated.
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
    ///
    /// let users_before = users::table.count().get_result::<i64>(&mut conn)?;
    /// let result =
    ///     profiles::table::try_builder()?.display_name("Negative").visits(-1).insert(&mut conn);
    /// assert!(matches!(result, Err(diesel_builders::BuilderError::Diesel(_))));
    /// let users_after = users::table.count().get_result::<i64>(&mut conn)?;
    /// assert_eq!(users_before, users_after);
    /// # Ok(())
    /// # }
    /// ```
    fn insert(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<Self::Table as TableExt>::Model, Self::ValidationError>;

    /// Atomically inserts all records and returns their nested model tuple.
    ///
    /// # Errors
    ///
    /// Returns an error if the insertion fails, if any remaining validation
    /// fails, or if any database constraints are violated.
    ///
    /// # Examples
    ///
    /// Insert the record and read back the nested model tuple.
    ///
    /// ```rust
    /// # include!("doctest_setup.rs");
    /// # use schema::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut conn = connection()?;
    /// let nested = children::table::builder()
    ///     .mandatory(sides::table::builder())
    ///     .discretionary(sides::table::builder())
    ///     .child_label("short")
    ///     .insert_nested(&mut conn)?;
    /// assert_eq!(nested.child_label(), "short");
    /// assert_eq!(children::table.count().get_result::<i64>(&mut conn)?, 1);
    /// # Ok(())
    /// # }
    /// ```
    fn insert_nested(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<<Self::Table as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels, Self::ValidationError>;
}
