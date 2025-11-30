//! Submodule defining a trait for creating default tuples.

use diesel_builders_macros::impl_default_tuple;

/// A trait for creating default values for tuple types.
pub trait DefaultTuple {
    /// Create a default instance of the tuple.
    fn default_tuple() -> Self;
}

// Generate implementations for all tuple sizes (1-32)
#[allow(clippy::unused_unit)]
#[impl_default_tuple]
mod impls {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tuple_unit() {
        <()>::default_tuple();
        assert_eq!((), ());
    }

    #[test]
    fn default_tuple_single() {
        let result = <(i32,)>::default_tuple();
        assert_eq!(result, (0,));
    }

    #[test]
    fn default_tuple_two() {
        let result = <(i32, String)>::default_tuple();
        assert_eq!(result, (0, String::new()));
    }

    #[test]
    fn default_tuple_three() {
        let result = <(i32, String, bool)>::default_tuple();
        assert_eq!(result, (0, String::new(), false));
    }

    #[test]
    fn default_tuple_mixed_types() {
        let result = <(Vec<i32>, Option<u8>, String)>::default_tuple();
        assert_eq!(result, (vec![], None, String::new()));
    }
}
