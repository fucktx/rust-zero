use core::service::ServiceConf;
use serde::{Deserialize, Serialize};

/// MiddlewaresConf：对齐 go-zero（默认都为 true）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MiddlewaresConf {
    #[serde(default = "default_true")]
    pub trace: bool,
    #[serde(default = "default_true")]
    pub log: bool,
    #[serde(default = "default_true")]
    pub prometheus: bool,
    #[serde(default = "default_true")]
    pub max_connections: bool,
    #[serde(default = "default_true")]
    pub breaker: bool,
    #[serde(default = "default_true")]
    pub shedding: bool,
    #[serde(default = "default_true")]
    pub timeout: bool,
    #[serde(default = "default_true")]
    pub recover: bool,
    #[serde(default = "default_true")]
    pub metrics: bool,
    #[serde(default = "default_true")]
    pub max_bytes: bool,
    #[serde(default = "default_true")]
    pub gunzip: bool,
}

fn default_true() -> bool {
    true
}

impl Default for MiddlewaresConf {
    fn default() -> Self {
        Self {
            trace: true,
            log: true,
            prometheus: true,
            max_connections: true,
            breaker: true,
            shedding: true,
            timeout: true,
            recover: true,
            metrics: true,
            max_bytes: true,
            gunzip: true,
        }
    }
}

/// PrivateKeyConf：对齐 go-zero。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PrivateKeyConf {
    #[serde(default)]
    pub fingerprint: String,
    #[serde(default)]
    pub key_file: String,
}

fn default_signature_strict() -> bool {
    false
}

/// go-zero 默认 `Expiry: 1h`，这里用秒表示（3600s）避免引入 Duration 反序列化额外依赖。
fn default_signature_expiry_secs() -> u64 {
    3600
}

/// SignatureConf：对齐 go-zero（保留语义）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SignatureConf {
    #[serde(default = "default_signature_strict")]
    pub strict: bool,
    /// 过期时间（秒），go-zero 为 `time.Duration` 默认 1h。
    #[serde(default = "default_signature_expiry_secs")]
    pub expiry_secs: u64,
    #[serde(default)]
    pub private_keys: Vec<PrivateKeyConf>,
}

impl Default for SignatureConf {
    fn default() -> Self {
        Self {
            strict: false,
            expiry_secs: 3600,
            private_keys: Vec::new(),
        }
    }
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}
fn default_port() -> u16 {
    8888
}
fn default_max_connections() -> i64 {
    10_000
}
fn default_max_bytes() -> i64 {
    1_048_576
}
fn default_timeout_ms() -> i64 {
    3_000
}
fn default_cpu_threshold() -> i64 {
    900
}

/// RestConf：对齐 go-zero rest.RestConf（定义应位于 rest 包内）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RestConf {
    /// 等价于 go-zero：`service.ServiceConf` 匿名嵌入
    #[serde(flatten, default)]
    pub service: ServiceConf,

    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cert_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_file: Option<String>,

    #[serde(default)]
    pub verbose: bool,

    #[serde(default = "default_max_connections")]
    pub max_connections: i64,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: i64,

    /// milliseconds
    #[serde(default = "default_timeout_ms")]
    pub timeout: i64,

    /// range: [0, 1000)
    #[serde(default = "default_cpu_threshold")]
    pub cpu_threshold: i64,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<SignatureConf>,

    /// There are default values for all the items in Middlewares.
    #[serde(default)]
    pub middlewares: MiddlewaresConf,

    /// TraceIgnorePaths is paths blacklist for trace middleware.
    #[serde(default)]
    pub trace_ignore_paths: Vec<String>,
}

impl Default for RestConf {
    fn default() -> Self {
        Self {
            service: ServiceConf::default(),
            host: default_host(),
            port: default_port(),
            cert_file: None,
            key_file: None,
            verbose: false,
            max_connections: default_max_connections(),
            max_bytes: default_max_bytes(),
            timeout: default_timeout_ms(),
            cpu_threshold: default_cpu_threshold(),
            signature: None,
            middlewares: MiddlewaresConf::default(),
            trace_ignore_paths: Vec::new(),
        }
    }
}

impl RestConf {
    pub fn addr_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn validate(&self) -> Result<(), String> {
        if !(0..1000).contains(&self.cpu_threshold) {
            return Err(format!(
                "CpuThreshold out of range [0,1000): {}",
                self.cpu_threshold
            ));
        }
        if self.port == 0 {
            return Err("Port must be > 0".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_should_work() {
        let c = RestConf::default();
        assert_eq!(c.host, "0.0.0.0");
        assert!(c.port > 0);
        assert_eq!(c.max_connections, 10_000);
        assert_eq!(c.max_bytes, 1_048_576);
        assert_eq!(c.timeout, 3_000);
        assert_eq!(c.cpu_threshold, 900);
        assert!(c.middlewares.trace);
        assert!(c.middlewares.gunzip);
    }
}
