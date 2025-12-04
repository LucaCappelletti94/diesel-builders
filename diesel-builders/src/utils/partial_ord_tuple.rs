//! Submodule defining a trait which allows comparing tuples for ordering.

use diesel_builders_macros::impl_partial_ord_tuple;

/// A trait for comparing tuples for ordering.
///
/// This trait provides a method to compare tuples for ordering.
pub trait PartialOrdTuple {
    /// Compare the tuple for ordering with another tuple.
    fn partial_cmp_tuple(&self, other: &Self) -> Option<std::cmp::Ordering>;
}

/// Generate implementations for all tuple sizes (0-32)
#[allow(clippy::unused_unit)]
#[impl_partial_ord_tuple]
mod impls {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partial_ord_tuple_unit() {
        let tuple1 = ();
        let tuple2 = ();
        // Call partial_cmp_tuple to test it compiles, result is always Some(Equal)
        assert_eq!(
            tuple1.partial_cmp_tuple(&tuple2),
            Some(std::cmp::Ordering::Equal)
        );
    }

    #[test]
    fn test_partial_ord_tuple_single() {
        let tuple1 = (42,);
        let tuple2 = (42,);
        let tuple3 = (43,);
        assert_eq!(
            tuple1.partial_cmp_tuple(&tuple2),
            Some(std::cmp::Ordering::Equal)
        );
        assert_eq!(
            tuple1.partial_cmp_tuple(&tuple3),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            tuple3.partial_cmp_tuple(&tuple1),
            Some(std::cmp::Ordering::Greater)
        );
    }

    #[test]
    fn test_partial_ord_tuple_two() {
        let tuple1 = (42, "hello".to_string());
        let tuple2 = (42, "hello".to_string());
        let tuple3 = (43, "hello".to_string());
        let tuple4 = (42, "world".to_string());
        assert_eq!(
            tuple1.partial_cmp_tuple(&tuple2),
            Some(std::cmp::Ordering::Equal)
        );
        assert_eq!(
            tuple1.partial_cmp_tuple(&tuple3),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            tuple3.partial_cmp_tuple(&tuple1),
            Some(std::cmp::Ordering::Greater)
        );
        assert_eq!(
            tuple1.partial_cmp_tuple(&tuple4),
            Some(std::cmp::Ordering::Less)
        );
    }

    #[test]
    fn test_partial_ord_tuple_three() {
        let tuple1 = (1, 2.5_f64, vec![1, 2, 3]);
        let tuple2 = (1, 2.5_f64, vec![1, 2, 3]);
        let tuple3 = (2, 2.5_f64, vec![1, 2, 3]);
        let tuple4 = (1, 3.5_f64, vec![1, 2, 3]);
        assert_eq!(
            tuple1.partial_cmp_tuple(&tuple2),
            Some(std::cmp::Ordering::Equal)
        );
        assert_eq!(
            tuple1.partial_cmp_tuple(&tuple3),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            tuple3.partial_cmp_tuple(&tuple1),
            Some(std::cmp::Ordering::Greater)
        );
        assert_eq!(
            tuple1.partial_cmp_tuple(&tuple4),
            Some(std::cmp::Ordering::Less)
        );
    }

    #[test]
    fn test_partial_ord_tuple_mixed_types() {
        let tuple1 = (1, "hello", 3.5);
        let tuple2 = (1, "hello", 3.5);
        let tuple3 = (2, "hello", 3.5);
        let tuple4 = (1, "world", 3.5);
        let tuple5 = (1, "hello", 2.71);
        assert_eq!(
            tuple1.partial_cmp_tuple(&tuple2),
            Some(std::cmp::Ordering::Equal)
        );
        assert_eq!(
            tuple1.partial_cmp_tuple(&tuple3),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            tuple3.partial_cmp_tuple(&tuple1),
            Some(std::cmp::Ordering::Greater)
        );
        assert_eq!(
            tuple1.partial_cmp_tuple(&tuple4),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            tuple1.partial_cmp_tuple(&tuple5),
            Some(std::cmp::Ordering::Greater)
        );
    }
}
