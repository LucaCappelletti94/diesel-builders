//! Submodule defining a trait which, for any tuple tuple (T1, T2, ...), defines
//! an associated type which is the `(Option<T1>, Option<T2>, ...)` tuple.

use diesel_builders_macros::impl_option_tuple;

use crate::{ClonableTuple, DebuggableTuple, DefaultTuple};

/// A trait for converting a tuple type into its corresponding option tuple
/// type.
pub trait OptionTuple {
    /// The associated option tuple type.
    type Output: DefaultTuple
        + TransposeOptionTuple<Transposed = Self>
        + ClonableTuple
        + DebuggableTuple;

    /// Convert the tuple into its optional variant.
    fn into_option(self) -> Self::Output;
}

/// A trait for transposing an option tuple into a tuple of options.
pub trait TransposeOptionTuple {
    /// The transposed tuple type.
    type Transposed: OptionTuple<Output = Self>;

    /// Transpose the option tuple into a tuple of options.
    fn transpose_option(self) -> Option<Self::Transposed>;
}

// Generate implementations for all tuple sizes (1-32)
#[impl_option_tuple]
mod impls {}
