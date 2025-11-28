//! Submodule defining a trait which allows cloning a tuple of any size.

use diesel_builders_macros::impl_clonable_tuple;

/// A trait for cloning tuples.
///
/// This trait provides a method to clone a tuple by cloning each element.
pub trait ClonableTuple {
    /// Clone the tuple by cloning each element.
    fn clone_tuple(&self) -> Self;
}

/// Generate implementations for all tuple sizes (0-32)
#[impl_clonable_tuple]
mod impls {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clone_tuple_unit() {
        let tuple = ();
        let cloned = tuple.clone_tuple();
        assert_eq!(tuple, cloned);
    }

    #[test]
    fn test_clone_tuple_single() {
        let tuple = (42,);
        let cloned = tuple.clone_tuple();
        assert_eq!(tuple, cloned);
        assert_eq!(cloned.0, 42);
    }

    #[test]
    fn test_clone_tuple_two() {
        let tuple = (42, "hello".to_string());
        let cloned = tuple.clone_tuple();
        assert_eq!(tuple.0, cloned.0);
        assert_eq!(tuple.1, cloned.1);
    }

    #[test]
    fn test_clone_tuple_three() {
        let tuple = (1, 2.5, vec![1, 2, 3]);
        let cloned = tuple.clone_tuple();
        assert_eq!(tuple.0, cloned.0);
        assert_eq!(tuple.1, cloned.1);
        assert_eq!(tuple.2, cloned.2);
    }

    #[test]
    fn test_clone_tuple_mixed_types() {
        let tuple = (42i32, "test".to_string(), vec![1, 2, 3], Some(100u64));
        let cloned = tuple.clone_tuple();
        assert_eq!(tuple.0, cloned.0);
        assert_eq!(tuple.1, cloned.1);
        assert_eq!(tuple.2, cloned.2);
        assert_eq!(tuple.3, cloned.3);
    }
}
