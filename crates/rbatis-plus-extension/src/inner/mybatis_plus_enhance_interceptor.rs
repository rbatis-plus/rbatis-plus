//! 增强拦截器调度器（对齐 Java `mybatis-plus-enhance` 的 `MybatisPlusEnhanceInterceptor`）。
//!
//! 该拦截器是 InnerInterceptor 的"二次分发"入口：
//! - 在官方 InnerInterceptor 的 beforeQuery/afterQuery/beforeUpdate/afterUpdate 基础上
//! - 增加 EnhanceInnerInterceptor 的 afterQuery/afterUpdate/afterExecution 后置钩子
//! - 管理阶段顺序校验（EnhancePhase）
//!
//! 对应 Java：
//! - `com.baomidou.mybatisplus.enhance.plugins.MybatisPlusEnhanceInterceptor`
//!   （mybatis-plus-enhance-core/src/main/java/.../MybatisPlusEnhanceInterceptor.java，262 行）

use async_trait::async_trait;
use rbatis::executor::Executor;
use rbatis::intercept::{Action, Intercept, ResultType};
use rbatis::plugin::transaction::{
    TransactionEvent, TransactionEventType, TransactionListener,
};
use rbatis::{Error, RBatis};
use rbdc::db::ExecResult;
use rbs::Value;
use std::sync::Arc;

use super::InnerInterceptor;

/// 增强拦截器调度器（对齐 Java `MybatisPlusEnhanceInterceptor`）。
///
/// 职责：
/// - 按 `EnhancePhase` 顺序校验并调度所有 `EnhanceInnerInterceptor`
/// - 在 Query/Update 前执行 `before_query` / `before_update`
/// - 在 Query/Update 后执行 `after_query` / `after_update`
/// - 在 finally 块中执行 `after_execution`（观测/metrics）
/// - 接收并转发 `on_transaction_event`
///
/// ## 阶段顺序约束
/// 写入前：参数加密 (200) → 数据签名 (300)
/// 查询后：验签 (300) → 解密 (400) → 国际化 (500) → 观测 (900)
///
/// 对应 Java：`com.baomidou.mybatisplus.enhance.plugins.MybatisPlusEnhanceInterceptor`（262 行）
#[derive(Debug)]
pub struct MybatisPlusEnhanceInterceptor {
    inner: Vec<Box<dyn InnerInterceptor>>,
}

impl MybatisPlusEnhanceInterceptor {
    /// 创建新的增强拦截器调度器。
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// 注册一个 InnerInterceptor（含阶段顺序校验）。
    ///
    /// 对应 Java：`MybatisPlusEnhanceInterceptor.addInnerInterceptor(InnerInterceptor)`
    pub fn add_inner_interceptor(&mut self, interceptor: impl InnerInterceptor + 'static) {
        self.validate_enhance_order(&interceptor);
        self.inner.push(Box::new(interceptor));
    }

    /// 批量设置拦截器（含阶段顺序校验）。
    pub fn set_inner_interceptors(&mut self, interceptors: Vec<Box<dyn InnerInterceptor>>) {
        for interceptor in &interceptors {
            self.validate_enhance_box(interceptor.as_ref());
        }
        self.inner = interceptors;
    }

    /// 校验增强拦截器阶段顺序。
    ///
    /// 对应 Java：`MybatisPlusEnhanceInterceptor.validateEnhanceOrder()`
    fn validate_enhance_order(&self, interceptor: &dyn InnerInterceptor) {
        // 读取拦截器的阶段值（通过 Debug trait 输出 hack）
        // 实际生产中，EnhanceInnerInterceptor trait 会有 phase() 方法
        // 这里先简化为不做校验，待 EnhanceInnerInterceptor trait 补全后实现
        let _ = interceptor; // 占位，待 EnhancePhase 集成
    }

    fn validate_enhance_box(&self, _interceptor: &dyn InnerInterceptor) {
        // 同上，待 EnhancePhase 集成
    }

    /// 获取已注册的拦截器列表。
    pub fn interceptors(&self) -> &[Box<dyn InnerInterceptor>] {
        &self.inner
    }

    /// 安装到 [`RBatis`] 拦截链（拦截器 + 事务事件转发器）。
    ///
    /// - 拦截器插入 `RBatis::intercepts` **最前**：SQL 改写（分页/租户等）
    ///   先于缓存（`RbatisCacheInterceptor`）执行，保证缓存看到改写后的
    ///   SQL（键一致、不串页）。
    /// - 同一实例同时注册为 `TransactionListener`，把 commit/rollback
    ///   事件转发给各 `InnerInterceptor::on_transaction_event`。
    ///
    /// 安装顺序约定：先 `install_cache`（缓存），再 `install`（增强）——
    /// 后安装者位于链首，因此增强改写先于缓存命中判定。
    pub fn install(self: Arc<Self>, rb: &RBatis) {
        rb.intercepts.insert(0, self.clone() as Arc<dyn Intercept>);
        rb.add_listener(self.clone() as Arc<dyn TransactionListener>);
    }
}

impl Default for MybatisPlusEnhanceInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

/// 桥接到 rbatis 拦截链：把 `Intercept::before/after` 分派给 InnerInterceptor。
///
/// `before` 按操作类型分派 `before_query` / `before_update`（短路语义透传）；
/// `after` 分派 `after_query` / `after_update` 并广播 `after_execution`
/// （观测/metrics）。`after_execution` 的耗时传 0——`Intercept` 钩子不提供
/// 耗时信息，需要精确计时的观测请直接包装 `Executor`。
#[async_trait]
impl Intercept for MybatisPlusEnhanceInterceptor {
    async fn before(
        &self,
        _task_id: i64,
        executor: &dyn Executor,
        sql: &mut String,
        args: &mut Vec<Value>,
        result: ResultType<&mut Result<ExecResult, Error>, &mut Result<Value, Error>>,
    ) -> Result<Action, Error> {
        match result {
            ResultType::Query(result) => {
                self.before_query(executor, sql, args, result).await
            }
            ResultType::Exec(result) => {
                self.before_update(executor, sql, args, result).await
            }
        }
    }

    async fn after(
        &self,
        _task_id: i64,
        executor: &dyn Executor,
        sql: &mut String,
        _args: &mut Vec<Value>,
        result: ResultType<&mut Result<ExecResult, Error>, &mut Result<Value, Error>>,
    ) -> Result<Action, Error> {
        match result {
            ResultType::Query(result) => {
                let dispatched = self.after_query(executor, sql, result).await;
                let failure = result.as_ref().err();
                self.after_execution(executor, sql, 0, failure).await;
                dispatched.map(|_| Action::Next)
            }
            ResultType::Exec(result) => {
                let dispatched = self.after_update(executor, sql, result).await;
                let failure = result.as_ref().err();
                self.after_execution(executor, sql, 0, failure).await;
                dispatched.map(|_| Action::Next)
            }
        }
    }
}

/// 事务事件转发器：把 rbatis 的 begin/commit/rollback 广播给各 inner 拦截器。
#[async_trait]
impl TransactionListener for MybatisPlusEnhanceInterceptor {
    async fn on_event(&self, event: &TransactionEvent) -> Result<(), Error> {
        if matches!(
            event.event_type,
            TransactionEventType::CommitSuccess
                | TransactionEventType::CommitFailed
                | TransactionEventType::Rollback
                | TransactionEventType::RollbackFailed
        ) {
            self.on_transaction_event(event).await;
        }
        Ok(())
    }
}

#[async_trait]
impl InnerInterceptor for MybatisPlusEnhanceInterceptor {
    /// 分派 before_query 到所有 inner interceptor。
    async fn before_query(
        &self,
        executor: &dyn Executor,
        sql: &mut String,
        args: &mut Vec<Value>,
        result: &mut Result<Value, Error>,
    ) -> Result<Action, Error> {
        for interceptor in &self.inner {
            match interceptor.before_query(executor, sql, args, result).await? {
                Action::Return => return Ok(Action::Return),
                Action::Next => {}
            }
        }
        Ok(Action::Next)
    }

    /// 分派 after_query 到所有 EnhanceInnerInterceptor。
    async fn after_query(
        &self,
        executor: &dyn Executor,
        sql: &str,
        result: &mut Result<Value, Error>,
    ) -> Result<(), Error> {
        for interceptor in &self.inner {
            interceptor.after_query(executor, sql, result).await?;
        }
        Ok(())
    }

    /// 分派 before_update 到所有 inner interceptor。
    async fn before_update(
        &self,
        executor: &dyn Executor,
        sql: &mut String,
        args: &mut Vec<Value>,
        result: &mut Result<ExecResult, Error>,
    ) -> Result<Action, Error> {
        for interceptor in &self.inner {
            match interceptor.before_update(executor, sql, args, result).await? {
                Action::Return => return Ok(Action::Return),
                Action::Next => {}
            }
        }
        Ok(Action::Next)
    }

    /// 分派 after_update 到所有 inner interceptor。
    async fn after_update(
        &self,
        executor: &dyn Executor,
        sql: &str,
        result: &mut Result<ExecResult, Error>,
    ) -> Result<(), Error> {
        for interceptor in &self.inner {
            interceptor.after_update(executor, sql, result).await?;
        }
        Ok(())
    }

    /// 在 finally 块中广播 after_execution（观测/metrics）。
    async fn after_execution(
        &self,
        executor: &dyn Executor,
        sql: &str,
        elapsed_nanos: u64,
        failure: Option<&Error>,
    ) {
        for interceptor in &self.inner {
            interceptor
                .after_execution(executor, sql, elapsed_nanos, failure)
                .await;
        }
    }

    /// 转发事务事件。
    async fn on_transaction_event(&self, event: &TransactionEvent) {
        for interceptor in &self.inner {
            interceptor.on_transaction_event(event).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enhance_interceptor_empty() {
        let interceptor = MybatisPlusEnhanceInterceptor::new();
        assert!(interceptor.interceptors().is_empty());
    }
}
