//! SQL 观察模块（对标 mybatis-plus-enhance `observation` 包）。
//!
//! 提供 SQL 执行时间观察、慢查询告警等能力。

use std::time::Duration;

/// SQL 观察处理器 trait（对标 Java `SqlObservationHandler`）。
pub trait SqlObservationHandler: Send + Sync + 'static {
    /// 记录 SQL 执行。
    fn on_query(&self, sql: &str, elapsed: Duration);

    /// 记录 SQL 异常。
    fn on_error(&self, sql: &str, error: &str, elapsed: Duration);

    /// 慢查询阈值（默认 1000ms）。
    fn slow_query_threshold(&self) -> Duration {
        Duration::from_millis(1000)
    }
}

/// 默认 SQL 观察处理器（日志输出）。
#[derive(Debug, Clone)]
pub struct DefaultObservationHandler {
    slow_threshold: Duration,
}

impl Default for DefaultObservationHandler {
    fn default() -> Self {
        Self { slow_threshold: Duration::from_millis(1000) }
    }
}

impl DefaultObservationHandler {
    pub fn new(threshold: Duration) -> Self {
        Self { slow_threshold: threshold }
    }
}

impl SqlObservationHandler for DefaultObservationHandler {
    fn on_query(&self, sql: &str, elapsed: Duration) {
        if elapsed >= self.slow_threshold {
            log::warn!("[慢查询] 耗时 {:?}: {}", elapsed, &sql[..sql.len().min(200)]);
        } else {
            log::debug!("[SQL] 耗时 {:?}: {}", elapsed, &sql[..sql.len().min(100)]);
        }
    }

    fn on_error(&self, sql: &str, error: &str, elapsed: Duration) {
        log::error!("[SQL异常] 耗时 {:?}: {} | 错误: {}", elapsed, &sql[..sql.len().min(200)], error);
    }

    fn slow_query_threshold(&self) -> Duration {
        self.slow_threshold
    }
}
