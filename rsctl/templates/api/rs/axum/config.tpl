// placeholder (axum) - reuse `templates/api/rs/config.tpl`

// Code scaffolded by rsctl. Safe to edit.
// rsctl {{.version}}

use serde::Deserialize;
use rest::RestConf; // 提前实现好的公共配置结构体
use anyhow::Context;
use std::fs;
{{.extraImports}}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub rest: RestConf
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rest: RestConf::default()
        }
    }
}

pub fn load_config() -> Config {
    // TODO: 之后可以改成从环境变量 / 配置文件加载
    Config::default()
}

/// 从 YAML 配置文件加载。
pub fn load(path: &str) -> anyhow::Result<Config> {
    let raw = fs::read_to_string(path).with_context(|| format!("read config: {path}"))?;
    let cfg: Config = serde_yaml::from_str(&raw).context("parse yaml config")?;
    Ok(cfg)
}


