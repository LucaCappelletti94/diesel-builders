//! Utility modules for Diesel relations.

pub mod option_tuple;
pub use option_tuple::{OptionTuple, TransposeOptionTuple};
pub mod default_tuple;
pub use default_tuple::DefaultTuple;
pub mod ref_tuple;
pub use ref_tuple::RefTuple;
pub mod clonable_tuple;
pub use clonable_tuple::ClonableTuple;
pub mod copiable_tuple;
pub use copiable_tuple::CopiableTuple;
pub mod partial_eq_tuple;
pub use partial_eq_tuple::PartialEqTuple;
pub mod debuggable_tuple;
pub use debuggable_tuple::DebuggableTuple;
