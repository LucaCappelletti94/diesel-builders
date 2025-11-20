//! Utility modules for Diesel relations.

#[macro_use]
pub mod tuple_impls;
pub mod extend_tuple;
pub use extend_tuple::ExtendTuple;
pub mod option_tuple;
pub use option_tuple::OptionTuple;
