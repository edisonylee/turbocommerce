// Test that #[page] without a path argument fails to compile.

use turbo_macros::page;

#[page]
fn BadPage() -> String {
    "Bad".to_string()
}

fn main() {}
