//! Compile-time tests for diesel-builders-macros using trybuild.
//!
//! This module tests that our macros:
//! - Compile successfully when used correctly (tests/ui/pass/)
//! - Fail to compile with appropriate errors when misused (tests/ui/fail/)

#[test]
fn ui_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass/*.rs");
}

#[test]
fn ui_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail/*.rs");
}
