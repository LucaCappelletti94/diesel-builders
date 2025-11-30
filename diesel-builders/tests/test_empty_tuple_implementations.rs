//! Tests for empty tuple implementations that exist for compile-time trait
//! satisfaction but are never actually called in practice.
//!
//! These implementations are necessary for the type system to work correctly
//! when working with empty tuples `()`, but they're typically never invoked
//! at runtime. This test module ensures they work correctly and are covered
//! by test coverage.

use diesel_builders::{
    GetColumns, MayGetColumns, MaySetColumns, SetColumns, SetHomogeneousColumn, TryMaySetColumns,
    TrySetColumns, TrySetHomogeneousColumn,
};

/// A simple test struct to use with the empty tuple trait implementations.
#[derive(Debug, Default)]
struct TestBuilder {
    data: String,
}

#[test]
fn test_get_columns_empty_tuple() {
    let builder = TestBuilder::default();
    // Call the empty tuple implementation
    GetColumns::<()>::get_columns(&builder);
}

#[test]
fn test_may_get_columns_empty_tuple() {
    let builder = TestBuilder::default();
    // Call the empty tuple implementation
    MayGetColumns::<()>::may_get_columns(&builder);
}

#[test]
fn test_set_columns_empty_tuple() {
    let mut builder = TestBuilder::default();
    // Call the empty tuple implementation
    let result = SetColumns::<()>::set_columns(&mut builder, ());
    assert_eq!(result.data, "");
}

#[test]
fn test_may_set_columns_empty_tuple() {
    let mut builder = TestBuilder::default();
    // Call the empty tuple implementation
    let result = MaySetColumns::<()>::may_set_columns(&mut builder, ());
    assert_eq!(result.data, "");
}

#[test]
fn test_set_homogeneous_columns_empty_tuple() {
    let mut builder = TestBuilder::default();
    let value = "test";
    // Call the empty tuple implementation
    let result = SetHomogeneousColumn::<&str, ()>::set_homogeneous_columns(&mut builder, &value);
    assert_eq!(result.data, "");
}

#[test]
fn test_try_set_columns_empty_tuple() {
    let mut builder = TestBuilder::default();
    // Call the empty tuple implementation
    let result = TrySetColumns::<()>::try_set_columns(&mut builder, ());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().data, "");
}

#[test]
fn test_try_may_set_columns_empty_tuple() {
    let mut builder = TestBuilder::default();
    // Call the empty tuple implementation
    let result = TryMaySetColumns::<()>::try_may_set_columns(&mut builder, ());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().data, "");
}

#[test]
fn test_try_set_homogeneous_columns_empty_tuple() {
    let mut builder = TestBuilder::default();
    let value = 42;
    // Call the empty tuple implementation
    let result =
        TrySetHomogeneousColumn::<i32, ()>::try_set_homogeneous_columns(&mut builder, &value);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().data, "");
}

#[test]
fn test_chained_empty_tuple_operations() {
    let mut builder = TestBuilder {
        data: "initial".to_string(),
    };

    // Test chaining multiple empty tuple operations
    let result = SetColumns::<()>::set_columns(&mut builder, ());
    let result = MaySetColumns::<()>::may_set_columns(result, ());
    let result = SetHomogeneousColumn::<i32, ()>::set_homogeneous_columns(result, &123);

    assert_eq!(result.data, "initial");
}

#[test]
fn test_chained_try_empty_tuple_operations() {
    let mut builder = TestBuilder {
        data: "initial".to_string(),
    };

    // Test chaining multiple empty tuple try operations
    let result = TrySetColumns::<()>::try_set_columns(&mut builder, ())
        .and_then(|b| TryMaySetColumns::<()>::try_may_set_columns(b, ()))
        .and_then(|b| {
            TrySetHomogeneousColumn::<String, ()>::try_set_homogeneous_columns(
                b,
                &"test".to_string(),
            )
        });

    assert!(result.is_ok());
    assert_eq!(result.unwrap().data, "initial");
}
