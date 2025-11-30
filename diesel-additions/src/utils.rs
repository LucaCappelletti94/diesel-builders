//! Utility modules for Diesel relations.

pub mod option_tuple;
pub use option_tuple::{OptionTuple, TransposeOptionTuple};
pub mod default_tuple;
pub use default_tuple::DefaultTuple;
pub mod ref_tuple;
pub use ref_tuple::RefTuple;
pub mod clonable_tuple;
pub use clonable_tuple::ClonableTuple;
pub mod debuggable_tuple;
pub use debuggable_tuple::DebuggableTuple;
