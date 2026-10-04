//! Submodule defining the `TableModel` trait.

use tuplities::prelude::NestedTupleInto;

use crate::{
    DescendantWithSelf, GetNestedColumns, HasTableExt, NestedColumns, NestedModel, TableExt,
    TypedNestedTuple, ancestors::DescendantOfAll, load_nested_query_builder::LoadNestedFirst,
};

/// A marker trait for Diesel table models.
///
/// This trait indicates that a type represents a Diesel table model and
/// provides access to the associated table type. It is automatically
/// implemented for any type that has a table with the required extensions.
///
/// Extends [`HasTableExt`] and [`Sized`].
///
/// This trait is typically derived automatically via the
/// `#[derive(TableModel)]` macro on your model structs.
pub trait TableModel: HasTableExt<Table: TableExt<Model = Self>> + Sized + Clone {
    /// Loads the full nested model for this table model (including all
    /// ancestors).
    ///
    /// # Errors
    ///
    /// Returns a [`diesel::result::Error`] if the database query fails.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::TableModel;
    /// use schema::*;
    ///
    /// let mut conn = connection_with_data()?;
    /// let profile = Profile::find(&1, &mut conn)?;
    /// let (user, (profile,)) = profile.nested(&mut conn)?;
    /// assert_eq!(user.id, profile.id);
    /// assert_eq!(user.name, profile.display_name);
    /// assert_eq!(profile.visits, 3);
    ///
    /// let ghost = Post { id: 42, user_id: 1, title: "Ghost".to_owned() };
    /// let missing = ghost.nested(&mut conn);
    /// assert!(matches!(missing, Err(diesel::result::Error::NotFound)));
    /// # Ok(())
    /// # }
    /// ```
    fn nested<Conn>(
        &self,
        conn: &mut Conn,
    ) -> diesel::QueryResult<NestedModel<Self::Table>>
    where
        Self::Table: DescendantWithSelf
            + DescendantOfAll<
                <<Self::Table as TableExt>::NestedPrimaryKeyColumns as NestedColumns>::NestedTables,
            >,
        Self: GetNestedColumns<<Self::Table as TableExt>::NestedPrimaryKeyColumns>,
        <Self::Table as TableExt>::NestedPrimaryKeyColumns: LoadNestedFirst<Self::Table, Conn>,
        <<Self::Table as TableExt>::NestedPrimaryKeyColumns as TypedNestedTuple>::NestedTupleValueType:
            NestedTupleInto<
                <<Self::Table as TableExt>::NestedPrimaryKeyColumns as TypedNestedTuple>::NestedTupleValueType,
            >,
{
        let pk = self.get_nested_columns();
        <<Self::Table as TableExt>::NestedPrimaryKeyColumns as LoadNestedFirst<
            Self::Table,
            Conn,
        >>::load_nested_first(pk, conn)
    }
}

impl<T> TableModel for T where T: HasTableExt<Table: TableExt<Model = T>> + Clone {}
