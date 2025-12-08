//! Submodule providing the `Typed` trait.

/// Trait representing an object with an associated type.
pub trait Typed {
    /// The Rust type associated with this object, as it appears in queries.
    type Type;
}
