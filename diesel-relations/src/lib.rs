#![doc = include_str!("../README.md")]

#[macro_use]
extern crate diesel_additions;

pub mod ancestors;
pub mod horizontal_same_as;
pub mod vertical_same_as;
pub mod vertical_same_as_group;
pub use horizontal_same_as::{HorizontalSameAsColumn, HorizontalSameAsKey, HorizontalSameAsKeys};
