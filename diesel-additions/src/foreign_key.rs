//! Submodule defining a `ForeignKey` trait for Diesel tables.

use crate::Key;

/// A trait for Diesel tables that define foreign key relationships.
pub trait ForeignKey<ReferencedColumns: Key>: Key {}
