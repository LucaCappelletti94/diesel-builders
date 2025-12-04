//! Submodule defining a trait which allows comparing tuples for equality.

use diesel_builders_macros::impl_partial_eq_tuple;

/// A trait for comparing tuples for equality.
///
/// This trait provides a method to compare tuples for equality.
pub trait PartialEqTuple {
    /// Compare the tuple for equality with another tuple.
    fn partial_eq_tuple(&self, other: &Self) -> bool;
}

/// Generate implementations for all tuple sizes (0-32)
#[allow(clippy::unused_unit)]
#[impl_partial_eq_tuple]
mod impls {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partial_e_tuple_unit() {
        let tuple1 = ();
        let tuple2 = ();
        // Call partial_eq_tuple to test it compiles, result is always true
        assert!(tuple1.partial_eq_tuple(&tuple2));
    }

    #[test]
    fn test_partial_eq_tuple_single() {
        let tuple1 = (42,);
        let tuple2 = (42,);
        let tuple3 = (43,);
        assert!(tuple1.partial_eq_tuple(&tuple2));
        assert!(!tuple1.partial_eq_tuple(&tuple3));
    }

    #[test]
    fn test_partial_eq_tuple_two() {
        let tuple1 = (42, "hello".to_string());
        let tuple2 = (42, "hello".to_string());
        let tuple3 = (43, "hello".to_string());
        assert!(tuple1.partial_eq_tuple(&tuple2));
        assert!(!tuple1.partial_eq_tuple(&tuple3));
    }

    #[test]
    fn test_partial_eq_tuple_three() {
        let tuple1 = (1, 2.5_f64, vec![1, 2, 3]);
        let tuple2 = (1, 2.5_f64, vec![1, 2, 3]);
        let tuple3 = (1, 2.6_f64, vec![1, 2, 3]);
        assert!(tuple1.partial_eq_tuple(&tuple2));
        assert!(!tuple1.partial_eq_tuple(&tuple3));
    }

    #[test]
    fn test_partial_eq_tuple_mixed_types() {
        let tuple1 = (42i32, "test".to_string(), vec![1, 2, 3], Some(100u64));
        let tuple2 = (42i32, "test".to_string(), vec![1, 2, 3], Some(100u64));
        let tuple3 = (42i32, "test".to_string(), vec![1, 2, 3], Some(101u64));
        assert!(tuple1.partial_eq_tuple(&tuple2));
        assert!(!tuple1.partial_eq_tuple(&tuple3));
    }
}
