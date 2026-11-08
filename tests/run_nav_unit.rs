// Run with: rustc --edition=2024 --test tests/run_nav_unit.rs -o nav-test
// Set up next to the patched Horizon tree or update this relative path.
#[path = "../horizon-0.2.92/crates/horizon-client/src/game/impact_native_nav.rs"]
mod nav;
