//! Tests for `NewCatError` and `NewCat` validation.

mod common;

use common::{NewCatError, cats};
use diesel_builders::TableExt;

#[test]
fn new_cat_error_display() {
    assert_eq!(
        format!("{}", NewCatError::ColorEmpty),
        "Color cannot be empty"
    );
}

#[test]
fn new_cat_try_set_column_empty_color() {
    let mut nc = <cats::table as TableExt>::NewValues::default();

    let res = <<cats::table as TableExt>::NewValues as diesel_builders::TrySetColumn<
        cats::color,
    >>::try_set_column(&mut nc, String::new());

    assert_eq!(res.unwrap_err(), NewCatError::ColorEmpty);

    // whitespace-only should also fail
    let mut nc2 = <cats::table as TableExt>::NewValues::default();
    let res2 = <<cats::table as TableExt>::NewValues as diesel_builders::TrySetColumn<
        cats::color,
    >>::try_set_column(&mut nc2, "  \t \n".to_string());
    assert_eq!(res2.unwrap_err(), NewCatError::ColorEmpty);
}
