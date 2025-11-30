#![doc = include_str!("../README.md")]

pub mod ancestors;
pub mod horizontal_same_as;
pub mod vertical_same_as;
pub mod vertical_same_as_group;
pub use ancestors::{AncestorOfIndex, Descendant, DescendantOf, Root};
pub use horizontal_same_as::{
    DiscretionarySameAsIndex, HorizontalSameAsColumn, HorizontalSameAsKey, HorizontalSameAsKeys,
    MandatorySameAsIndex,
};
pub mod horizontal_same_as_group;
pub use horizontal_same_as_group::HorizontalSameAsGroup;
