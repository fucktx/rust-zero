// Code scaffolded by rsctl. Safe to edit.
// rsctl {{.version}}

use rust_zero::rest::RestConf;
use serde::Deserialize;

{{.jwtConf}}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub rest: RestConf,
    {{.jwtField}}
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rest: RestConf::default(),
            {{.jwtDefault}}
        }
    }
}
