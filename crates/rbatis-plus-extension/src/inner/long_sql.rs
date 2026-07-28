//! 超长 SQL 检测增强拦截器（对标 Java `LongSqlInnerInterceptor`）。
//!
//! 在 SQL 执行前检测 SQL 长度是否超过阈值，超过时调用 `LongSqlHandler` 处理。
//! 阶段：`OBSERVATION`（900），与观测拦截器同阶段。
//!
//! # 对应 Java
//!
//! - `com.baomidou.mybatisplus.enhance.plugins.inner.LongSqlInnerInterceptor`
//!   （mybatis-plus-enhance-extension/.../plugins/inner/LongSqlInnerInterceptor.java）

use async_trait::async_trait;
use rbatis::executor::Executor;
use rbatis::intercept::Action;
use rbatis::Error;
use rbdc::db::ExecResult;
use rbs::Value;
use std::sync::Arc;

use crate::inner::enhance_interceptor::EnhanceInnerInterceptor;
use crate::inner::enhance_phase::EnhancePhase;
use crate::inner::inner_interceptor::InnerInterceptor;

/// 超长 SQL 处理器 trait（对标 Java `LongSqlHandler`）。
///
/// 业务可实现该接口自定义超长 SQL 的处理策略（如日志告警、拒绝执行等）。
///
/// # 对应 Java
///
/// - `com.baomidou.mybatisplus.enhance.plugins.inner.LongSqlHandler`
pub trait LongSqlHandler: Send + Sync + std::fmt::Debug {
    /// 处理超长 SQL。
    ///
    /// 返回 `true` 允许继续执行，返回 `false` 拒绝执行。
    fn handle(&self, sql: &str, length: usize, threshold: usize) -> bool;
}

/// 默认超长 SQL 处理器（日志告警，允许继续执行）。
///
/// # 对应 Java
///
/// - 默认实现（日志输出后放行）
#[derive(Debug, Clone)]
pub struct DefaultLongSqlHandler;

impl LongSqlHandler for DefaultLongSqlHandler {
    fn handle(&self, sql: &str, length: usize, threshold: usize) -> bool {
        log::warn!(
            "[LongSql] SQL 长度 {} 超过阈值 {}，前 200 字符: {}",
            length,
            threshold,
            &sql[..sql.len().min(200)]
        );
        true // 允许继续执行
    }
}

/// 超长 SQL 检测增强拦截器（对标 Java `LongSqlInnerInterceptor`）。
///
/// 职责：
/// - `before_query` / `before_update`：检测 SQL 长度，超过阈值时调用处理器
///
/// # 对应 Java
///
/// - `com.baomidou.mybatisplus.enhance.plugins.inner.LongSqlInnerInterceptor`
#[derive(Clone)]
pub struct LongSqlInnerInterceptor {
    /// SQL 长度阈值（字符数）。
    max_length: usize,
    /// 超长 SQL 处理器。
    handler: Arc<dyn LongSqlHandler>,
}

impl std::fmt::Debug for LongSqlInnerInterceptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LongSqlInnerInterceptor")
            .field("max_length", &self.max_length)
            .finish()
    }
}

impl LongSqlInnerInterceptor {
    /// 创建超长 SQL 拦截器（使用默认处理器）。
    ///
    /// 对应 Java：`LongSqlInnerInterceptor(int maxLength)`
    pub fn new(max_length: usize) -> Self {
        Self {
            max_length,
            handler: Arc::new(DefaultLongSqlHandler),
        }
    }

    /// 使用自定义处理器创建。
    pub fn with_handler(max_length: usize, handler: Box<dyn LongSqlHandler>) -> Self {
        Self {
            max_length,
            handler: Arc::from(handler),
        }
    }

    /// 获取 SQL 长度阈值。
    pub fn max_length(&self) -> usize {
        self.max_length
    }

    /// 设置 SQL 长度阈值。
    pub fn set_max_length(&mut self, max_length: usize) {
        self.max_length = max_length;
    }

    /// 检测 SQL 是否超长并调用处理器。
    ///
    /// 对应 Java：`LongSqlInnerInterceptor.checkLength(String sql)`
    fn check_sql_length(&self, sql: &str) -> Action {
        let length = sql.len();
        if length > self.max_length {
            let allowed = self.handler.handle(sql, length, self.max_length);
            if !allowed {
                return Action::Return;
            }
        }
        Action::Next
    }
}

#[async_trait]
impl InnerInterceptor for LongSqlInnerInterceptor {
    /// SELECT 前检测 SQL 长度。
    ///
    /// 对应 Java：`LongSqlInnerInterceptor.beforeQuery(...)`
    async fn before_query(
        &self,
        _executor: &dyn Executor,
        sql: &mut String,
        _args: &mut Vec<Value>,
        _result: &mut Result<Value, Error>,
    ) -> Result<Action, Error> {
        Ok(self.check_sql_length(sql))
    }

    /// INSERT/UPDATE/DELETE 前检测 SQL 长度。
    ///
    /// 对应 Java：`LongSqlInnerInterceptor.beforeUpdate(...)`
    async fn before_update(
        &self,
        _executor: &dyn Executor,
        sql: &mut String,
        _args: &mut Vec<Value>,
        _result: &mut Result<ExecResult, Error>,
    ) -> Result<Action, Error> {
        Ok(self.check_sql_length(sql))
    }
}

#[async_trait]
impl EnhanceInnerInterceptor for LongSqlInnerInterceptor {
    /// 声明阶段为观测（900）。
    ///
    /// 对应 Java：`LongSqlInnerInterceptor.phase()`
    fn phase(&self) -> EnhancePhase {
        EnhancePhase::OBSERVATION
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// 测试用处理器，记录调用情况。
    #[derive(Debug)]
    struct MockLongSqlHandler {
        calls: Mutex<Vec<(String, usize, usize)>>,
        allow: bool,
    }

    impl MockLongSqlHandler {
        fn new(allow: bool) -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                allow,
            }
        }

        fn calls(&self) -> Vec<(String, usize, usize)> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl LongSqlHandler for MockLongSqlHandler {
        fn handle(&self, sql: &str, length: usize, threshold: usize) -> bool {
            self.calls
                .lock()
                .unwrap()
                .push((sql.to_string(), length, threshold));
            self.allow
        }
    }

    #[test]
    fn test_phase_is_observation() {
        let interceptor = LongSqlInnerInterceptor::new(1000);
        assert_eq!(interceptor.phase(), EnhancePhase::OBSERVATION);
    }

    #[test]
    fn test_short_sql_passes() {
        let handler = MockLongSqlHandler::new(true);
        let interceptor = LongSqlInnerInterceptor::with_handler(100, Box::new(handler));

        let sql = "SELECT 1";
        let action = interceptor.check_sql_length(sql);
        assert!(matches!(action, Action::Next));
    }

    #[test]
    fn test_long_sql_calls_handler() {
        let handler = MockLongSqlHandler::new(true);
        let interceptor = LongSqlInnerInterceptor::with_handler(10, Box::new(handler));

        let long_sql = "SELECT * FROM very_long_table_name WHERE id = 1";
        let action = interceptor.check_sql_length(long_sql);
        assert!(matches!(action, Action::Next));
    }

    #[test]
    fn test_long_sql_handler_can_reject() {
        let handler = MockLongSqlHandler::new(false);
        let interceptor = LongSqlInnerInterceptor::with_handler(10, Box::new(handler));

        let long_sql = "SELECT * FROM very_long_table_name WHERE id = 1";
        let action = interceptor.check_sql_length(long_sql);
        assert!(matches!(action, Action::Return));
    }

    #[test]
    fn test_max_length_getter_setter() {
        let mut interceptor = LongSqlInnerInterceptor::new(1000);
        assert_eq!(interceptor.max_length(), 1000);

        interceptor.set_max_length(2000);
        assert_eq!(interceptor.max_length(), 2000);
    }
}
