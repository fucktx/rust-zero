//! Stable IR (Spec) used by generators.

use serde::{Deserialize, Serialize};

pub mod api;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
}
