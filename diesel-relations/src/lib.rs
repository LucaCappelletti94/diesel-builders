#![doc = include_str!("../README.md")]

pub mod ancestors;
pub mod horizontal_same_as;
pub mod vertical_same_as;
pub mod vertical_same_as_group;
pub use ancestors::{AncestorOfIndex, Descendant, DescendantOf};
pub use horizontal_same_as::{HorizontalSameAsColumn, HorizontalSameAsKey, HorizontalSameAsKeys};
