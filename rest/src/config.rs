use serde::{Deserialize, Serialize};
use std::time;

struct Middlewares {
    trace: bool,
    log: bool,
    prometheus: bool,
    max_connections: bool,
    breaker: bool,
    shedding: bool,
    timeout: bool,
    recover: bool,
    metrics: bool,
    max_bytes: bool,
    gunzip: bool,
}
struct PrivateKey {
    fingerprint: String,
    keyfile: String,
}
struct Signature {
    strict: bool,
    expire: time::Duration,
    private_keys: Vec<PrivateKey>,
}
pub struct Rest {
    //	service.ServiceConf todo
    host: String,
    port: usize,
    cert_file: String,
    key_file: String,
    verbose: bool,
    max_connections: usize,
    max_bytes: usize,
    timeout: usize,
    cpu_threshold: usize,
    signature: Signature,
    middlewares: Middlewares,
    trace_ignore_paths: Vec<String>,
}
