//! Stable IR (Spec) used by generators.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
}


