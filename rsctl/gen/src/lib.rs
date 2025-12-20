//! Code generation: spec -> artifacts.

pub mod api;
pub mod artifact;
// version is configured in cli (see `cli/src/version.rs`)

pub fn ping() -> &'static str {
    "gen"
}


