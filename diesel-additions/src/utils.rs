//! Utility modules for Diesel relations.

#[macro_use]
pub mod tuple_impls;
pub mod option_tuple;
pub use option_tuple::{OptionTuple, TransposeOptionTuple};
pub mod default_tuple;
pub use default_tuple::DefaultTuple;
pub mod ref_tuple;
pub use ref_tuple::RefTuple;
