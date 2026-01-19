//! UI tests for compile-time errors.

#[rustversion::attr(nightly, ignore)]
#[test]
fn ui_compile_fail_nightly() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui_nightly/*.rs");
}

#[rustversion::attr(stable, ignore)]
#[test]
fn ui_compile_fail_stable() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui_stable/*.rs");
}
