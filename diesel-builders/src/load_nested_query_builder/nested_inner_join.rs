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
    fn nested_inner_join() -> Self::JoinQuery;
}
