//! Submodule defining a trait which allows copying a tuple of any size.

use diesel_builders_macros::impl_copiable_tuple;

/// A trait for copying tuples.
///
/// This trait provides a method to copy a tuple by copying each element.
pub trait CopiableTuple {
    /// Copy the tuple by copying each element.
    #[must_use]
    fn copy_tuple(&self) -> Self;
}

/// Generate implementations for all tuple sizes (0-32)
#[allow(clippy::unused_unit)]
#[impl_copiable_tuple]
mod impls {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copiable_tuple_unit() {
        let tuple = ();
        // Call copy_tuple to test it compiles, result is always ()
        assert_eq!(tuple.copy_tuple(), ());
    }

    #[test]
    fn test_copy_tuple_single() {
        let tuple = (42,);
        let copied = tuple.copy_tuple();
        assert_eq!(tuple, copied);
        assert_eq!(copied.0, 42);
    }

    #[test]
    fn test_copy_tuple_two() {
        let tuple = (42, "hello");
        let copied = tuple.copy_tuple();
        assert_eq!(tuple.0, copied.0);
        assert_eq!(tuple.1, copied.1);
    }

    #[test]
    fn test_copy_tuple_three() {
        let tuple = (1, 2.5_f64, [1, 2, 3]);
        let copied = tuple.copy_tuple();
        assert_eq!(tuple.0, copied.0);
        assert!((tuple.1 - copied.1).abs() < f64::EPSILON);
        assert_eq!(tuple.2, copied.2);
    }

    #[test]
    fn test_copy_tuple_mixed_types() {
        let tuple = (42i32, "test", [1, 2, 3], Some(100u64));
        let copied = tuple.copy_tuple();
        assert_eq!(tuple.0, copied.0);
        assert_eq!(tuple.1, copied.1);
        assert_eq!(tuple.2, copied.2);
        assert_eq!(tuple.3, copied.3);
    }
}
