//! Submodule providing a `NestedInnerJoin` trait which constructs an inner join
//! query for a table and all of its ancestors.

use crate::TableExt;

/// The `NestedInnerJoin` trait constructs the inner-join query over a table and
/// all of its ancestor tables.
///
/// It is implemented on the table itself by the `TableModel` derive macro,
/// where the concrete ancestor table types are known. Building the join there
/// means the recursive join kind marker (diesel's private `Inner`) never has to
/// be named in a generic bound written by this crate, and the join query type,
/// a foreign tuple-covered type, is defined in the crate that owns the tables
/// rather than through an orphan impl.
pub trait NestedInnerJoin: TableExt {
    /// The type of the constructed join query.
    type JoinQuery;

    /// Constructs an inner join query over the table and its ancestors.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::load_nested_query_builder::NestedInnerJoin;
    /// use schema::*;
    ///
    /// let mut conn = connection_with_data()?;
    /// let join = <profiles::table as NestedInnerJoin>::nested_inner_join();
    /// let rows: Vec<(Profile, User)> = join.load(&mut conn)?;
    /// assert!(
    ///     rows.iter()
    ///         .map(|(profile, user)| {
    ///             (
    ///                 profile.id,
    ///                 profile.display_name.as_str(),
    ///                 profile.visits,
    ///                 user.id,
    ///                 user.name.as_str(),
    ///                 user.age,
    ///                 user.nickname.as_deref(),
    ///             )
    ///         })
    ///         .eq([(1, "Ada", 3, 1, "Ada", 20, Some("Ace"))])
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn nested_inner_join() -> Self::JoinQuery;
}
