//! Submodule defining a trait which allows hashing tuples.

use diesel_builders_macros::impl_hash_tuple;

/// A trait for hashing tuples.
///
/// This trait provides a method to hash tuples element-wise.
pub trait HashTuple {
    /// Hash the tuple element-wise.
    fn hash_tuple<H: std::hash::Hasher>(&self, state: &mut H);
}

/// Generate implementations for all tuple sizes (0-32)
#[allow(clippy::unused_unit)]
#[impl_hash_tuple]
mod impls {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;

    #[test]
    fn test_hash_tuple_unit() {
        let tuple = ();
        let mut hasher = DefaultHasher::new();
        tuple.hash_tuple(&mut hasher);
        let hash1 = hasher.finish();

        let tuple2 = ();
        let mut hasher2 = DefaultHasher::new();
        tuple2.hash_tuple(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_tuple_single() {
        let tuple1 = (42,);
        let mut hasher1 = DefaultHasher::new();
        tuple1.hash_tuple(&mut hasher1);
        let hash1 = hasher1.finish();

        let tuple2 = (42,);
        let mut hasher2 = DefaultHasher::new();
        tuple2.hash_tuple(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);

        let tuple3 = (43,);
        let mut hasher3 = DefaultHasher::new();
        tuple3.hash_tuple(&mut hasher3);
        let hash3 = hasher3.finish();

        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hash_tuple_two() {
        let tuple1 = (42, "hello".to_string());
        let mut hasher1 = DefaultHasher::new();
        tuple1.hash_tuple(&mut hasher1);
        let hash1 = hasher1.finish();

        let tuple2 = (42, "hello".to_string());
        let mut hasher2 = DefaultHasher::new();
        tuple2.hash_tuple(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);

        let tuple3 = (43, "hello".to_string());
        let mut hasher3 = DefaultHasher::new();
        tuple3.hash_tuple(&mut hasher3);
        let hash3 = hasher3.finish();

        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hash_tuple_three() {
        let tuple1 = (1, 2_i32, vec![1, 2, 3]);
        let mut hasher1 = DefaultHasher::new();
        tuple1.hash_tuple(&mut hasher1);
        let hash1 = hasher1.finish();

        let tuple2 = (1, 2_i32, vec![1, 2, 3]);
        let mut hasher2 = DefaultHasher::new();
        tuple2.hash_tuple(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);

        let tuple3 = (1, 3_i32, vec![1, 2, 3]);
        let mut hasher3 = DefaultHasher::new();
        tuple3.hash_tuple(&mut hasher3);
        let hash3 = hasher3.finish();

        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hash_tuple_mixed_types() {
        let tuple1 = (42i32, "test".to_string(), vec![1, 2, 3], Some(100u64));
        let mut hasher1 = DefaultHasher::new();
        tuple1.hash_tuple(&mut hasher1);
        let hash1 = hasher1.finish();

        let tuple2 = (42i32, "test".to_string(), vec![1, 2, 3], Some(100u64));
        let mut hasher2 = DefaultHasher::new();
        tuple2.hash_tuple(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);

        let tuple3 = (42i32, "test".to_string(), vec![1, 2, 3], Some(101u64));
        let mut hasher3 = DefaultHasher::new();
        tuple3.hash_tuple(&mut hasher3);
        let hash3 = hasher3.finish();

        assert_ne!(hash1, hash3);
    }
}
