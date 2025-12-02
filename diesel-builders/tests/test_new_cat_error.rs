//! Tests for `NewCatError` and `NewCat` validation.

mod common;

use common::{NewCat, NewCatError, cats};

#[test]
fn new_cat_error_display() {
    assert_eq!(
        format!("{}", NewCatError::ColorEmpty),
        "Color cannot be empty"
    );
}

#[test]
fn new_cat_try_set_column_empty_color() {
    let mut nc = NewCat::default();

    let res = <NewCat as diesel_builders::TrySetColumn<cats::color>>::try_set_column(
        &mut nc,
        "".to_string(),
    );

    assert_eq!(res.unwrap_err(), NewCatError::ColorEmpty);

    // whitespace-only should also fail
    let mut nc2 = NewCat::default();
    let res2 = <NewCat as diesel_builders::TrySetColumn<cats::color>>::try_set_column(
        &mut nc2,
        "  \t \n".to_string(),
    );
    assert_eq!(res2.unwrap_err(), NewCatError::ColorEmpty);
}
