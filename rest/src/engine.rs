use crate::config::RestConf;
#[cfg(feature = "axum")]
use std::time::Duration;

/// 运行时引擎：持有 RestConf 与未来可扩展的运行时资源（日志、指标、连接池等）。
#[derive(Debug, Clone)]
pub struct Engine {
    pub conf: RestConf,
}

impl Engine {
    pub fn new(conf: RestConf) -> Self {
        Self { conf }
    }

    /// 类 go-zero：把 RestConf.Middlewares 中启用的“内置中间件”应用到 Router 上。
    ///
    /// 说明：
    /// - 这里仅实现当前 rest crate 已落地的、语义稳定的部分（timeout / max_bytes）。
    /// - 直接使用 axum/tower 原生 Layer，不引入额外运行时抽象成本。
    #[cfg(feature = "axum")]
    pub fn apply_defaults<S>(&self, mut router: ::axum::Router<S>) -> ::axum::Router<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        // WithTimeout：默认使用 conf.Timeout（毫秒）
        if self.conf.middlewares.timeout && self.conf.timeout > 0 {
            let duration = Duration::from_millis(self.conf.timeout as u64);
            router = router.layer(::axum::middleware::from_fn_with_state(
                crate::middleware::timeout::axum_timeout::state(duration),
                crate::middleware::timeout::axum_timeout::run,
            ));
        }

        // WithMaxBytes：默认使用 conf.MaxBytes
        if self.conf.middlewares.max_bytes {
            router = router.layer(crate::middleware::max_bytes::axum_max_bytes::layer(
                self.conf.max_bytes,
            ));
        }

        router
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
