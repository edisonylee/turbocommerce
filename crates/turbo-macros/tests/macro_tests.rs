/// Macro expansion tests using trybuild.
///
/// These tests verify that the procedural macros produce expected compile errors.
/// Note: Pass tests require full leptos/turbo-router dependencies and are better
/// tested via integration tests in the example storefront.

#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();

    // Fail tests - should fail with expected error messages
    t.compile_fail("tests/ui/page_missing_path.rs");
}
