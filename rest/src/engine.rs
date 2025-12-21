use crate::config::RestConf;

/// 运行时引擎：持有 RestConf 与未来可扩展的运行时资源（日志、指标、连接池等）。
#[derive(Debug, Clone)]
pub struct Engine {
    pub conf: RestConf,
}

impl Engine {
    pub fn new(conf: RestConf) -> Self {
        Self { conf }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_should_store_conf() {
        let conf = RestConf::default();
        let engine = Engine::new(conf.clone());
        assert_eq!(engine.conf.host, conf.host);
        assert_eq!(engine.conf.port, conf.port);
    }
}
