//! SQL 观测增强拦截器（对标 Java `SqlObservationInnerInterceptor`）。
//!
//! 在 SQL 执行完成后，构造 `SqlObservation` 并发送给所有注册的观测接收器（Sink）。
//! 阶段：`OBSERVATION`（900），在所有数据增强之后执行。
//!
//! # 对应 Java
//!
//! - `com.baomidou.mybatisplus.enhance.plugins.inner.SqlObservationInnerInterceptor`
//!   （mybatis-plus-enhance-extension/.../plugins/inner/SqlObservationInnerInterceptor.java，90 行）

use async_trait::async_trait;
use rbatis::executor::Executor;
use rbatis::Error;
use std::sync::Arc;

use crate::inner::enhance_interceptor::EnhanceInnerInterceptor;
use crate::inner::enhance_phase::EnhancePhase;
use crate::inner::inner_interceptor::InnerInterceptor;

/// SQL 观测数据（对标 Java `SqlObservation`）。
///
/// 封装单次 SQL 执行的观测信息，发送给 `SqlObservationSink` 处理。
///
/// # 对应 Java
///
/// - `com.baomidou.mybatisplus.enhance.plugins.observation.SqlObservation`
#[derive(Debug, Clone)]
pub struct SqlObservation {
    /// 执行的 SQL 语句。
    pub sql: String,
    /// 执行耗时（纳秒）。
    pub elapsed_nanos: u64,
    /// 执行是否失败（None 表示成功）。
    pub failure: Option<String>,
}

impl SqlObservation {
    /// 创建成功的观测。
    pub fn success(sql: impl Into<String>, elapsed_nanos: u64) -> Self {
        Self {
            sql: sql.into(),
            elapsed_nanos,
            failure: None,
        }
    }

    /// 创建失败的观测。
    pub fn failure(sql: impl Into<String>, elapsed_nanos: u64, error: impl Into<String>) -> Self {
        Self {
            sql: sql.into(),
            elapsed_nanos,
            failure: Some(error.into()),
        }
    }

    /// 执行是否成功。
    pub fn is_success(&self) -> bool {
        self.failure.is_none()
    }

    /// 获取执行耗时（毫秒）。
    pub fn elapsed_millis(&self) -> f64 {
        self.elapsed_nanos as f64 / 1_000_000.0
    }
}

/// SQL 观测接收器 trait（对标 Java `SqlObservationSink`）。
///
/// 业务可实现该接口接入 Prometheus、OpenTelemetry 等观测系统。
///
/// # 对应 Java
///
/// - `com.baomidou.mybatisplus.enhance.plugins.observation.SqlObservationSink`
pub trait SqlObservationSink: Send + Sync + std::fmt::Debug {
    /// 接收 SQL 观测数据。
    fn accept(&self, observation: &SqlObservation);
}

/// SQL 观测增强拦截器（对标 Java `SqlObservationInnerInterceptor`）。
///
/// 职责：
/// - `after_execution`：构造 `SqlObservation`，广播给所有注册的 Sink
///
/// # 对应 Java
///
/// - `com.baomidou.mybatisplus.enhance.plugins.inner.SqlObservationInnerInterceptor`
pub struct SqlObservationInnerInterceptor {
    /// 观测接收器列表。
    sinks: Vec<Arc<dyn SqlObservationSink>>,
}

impl std::fmt::Debug for SqlObservationInnerInterceptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqlObservationInnerInterceptor")
            .field("sink_count", &self.sinks.len())
            .finish()
    }
}

impl SqlObservationInnerInterceptor {
    /// 创建 SQL 观测拦截器。
    ///
    /// 对应 Java：`SqlObservationInnerInterceptor()`
    pub fn new() -> Self {
        Self { sinks: Vec::new() }
    }

    /// 注册一个观测接收器。
    ///
    /// 对应 Java：`SqlObservationInnerInterceptor.addSink(SqlObservationSink)`
    pub fn add_sink(&mut self, sink: impl SqlObservationSink + 'static) {
        self.sinks.push(Arc::new(sink));
    }

    /// 批量注册观测接收器。
    pub fn set_sinks(&mut self, sinks: Vec<Arc<dyn SqlObservationSink>>) {
        self.sinks = sinks;
    }

    /// 获取已注册的接收器数量。
    pub fn sink_count(&self) -> usize {
        self.sinks.len()
    }
}

#[async_trait]
impl InnerInterceptor for SqlObservationInnerInterceptor {
    /// SQL 执行完成后，构造观测数据并广播。
    ///
    /// 对应 Java：`SqlObservationInnerInterceptor.afterExecution(...)`
    async fn after_execution(
        &self,
        _executor: &dyn Executor,
        sql: &str,
        elapsed_nanos: u64,
        failure: Option<&Error>,
    ) {
        let observation = match failure {
            Some(err) => SqlObservation::failure(sql, elapsed_nanos, err.to_string()),
            None => SqlObservation::success(sql, elapsed_nanos),
        };

        for sink in &self.sinks {
            sink.accept(&observation);
        }
    }
}

#[async_trait]
impl EnhanceInnerInterceptor for SqlObservationInnerInterceptor {
    /// 声明阶段为观测（900）。
    ///
    /// 对应 Java：`SqlObservationInnerInterceptor.phase()`
    fn phase(&self) -> EnhancePhase {
        EnhancePhase::OBSERVATION
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// 测试用观测接收器，记录接收到的观测。
    #[derive(Debug)]
    struct MockSink {
        observations: Mutex<Vec<SqlObservation>>,
    }

    impl MockSink {
        fn new() -> Self {
            Self {
                observations: Mutex::new(Vec::new()),
            }
        }

        fn observations(&self) -> Vec<SqlObservation> {
            self.observations.lock().unwrap().clone()
        }
    }

    impl SqlObservationSink for MockSink {
        fn accept(&self, observation: &SqlObservation) {
            self.observations.lock().unwrap().push(observation.clone());
        }
    }

    #[test]
    fn test_phase_is_observation() {
        let interceptor = SqlObservationInnerInterceptor::new();
        assert_eq!(interceptor.phase(), EnhancePhase::OBSERVATION);
    }

    #[test]
    fn test_add_sink() {
        let mut interceptor = SqlObservationInnerInterceptor::new();
        assert_eq!(interceptor.sink_count(), 0);

        interceptor.add_sink(MockSink::new());
        assert_eq!(interceptor.sink_count(), 1);
    }

    #[test]
    fn test_sql_observation_success() {
        let obs = SqlObservation::success("SELECT 1", 1_500_000);
        assert!(obs.is_success());
        assert_eq!(obs.sql, "SELECT 1");
        assert!((obs.elapsed_millis() - 1.5).abs() < 0.01);
        assert!(obs.failure.is_none());
    }

    #[test]
    fn test_sql_observation_failure() {
        let obs = SqlObservation::failure("INSERT INTO t", 500_000, "duplicate key");
        assert!(!obs.is_success());
        assert_eq!(obs.failure.as_deref(), Some("duplicate key"));
    }

    #[test]
    fn test_sink_receives_observations() {
        let sink = Arc::new(MockSink::new());
        let mut interceptor = SqlObservationInnerInterceptor::new();
        interceptor.set_sinks(vec![sink.clone()]);

        let obs = SqlObservation::success("SELECT * FROM users", 2_000_000);
        // 模拟广播
        for s in &interceptor.sinks {
            s.accept(&obs);
        }

        let observations = sink.observations();
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].sql, "SELECT * FROM users");
    }
}
