//! Binary target for cargo-leptos (not used at runtime - Spin handles SSR).

fn main() {
    // This binary is only needed to satisfy cargo-leptos during build.
    // Actual server-side rendering is handled by Spin via server.rs
}
