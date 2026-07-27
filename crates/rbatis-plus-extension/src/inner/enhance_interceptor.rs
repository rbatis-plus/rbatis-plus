//! 增强拦截器 trait（对齐 Java `mybatis-plus-enhance` 的 `EnhanceInnerInterceptor`）。
//!
//! 在官方 InnerInterceptor 的 4 个 before/after 钩子基础上，
//! 增加 afterQuery / afterUpdate / afterExecution 三个后置钩子，
//! 以及阶段声明 `phase()` 方法。
//!
//! 对应 Java：
//! - `com.baomidou.mybatisplus.enhance.plugins.inner.EnhanceInnerInterceptor`
//!   （mybatis-plus-enhance-core/src/main/java/.../EnhanceInnerInterceptor.java，91 行）

use async_trait::async_trait;
use rbatis::executor::Executor;
use rbatis::Error;
use rbdc::db::ExecResult;
use rbs::Value;

use super::enhance_phase::EnhancePhase;

/// 增强拦截器 trait。
///
/// 对应 Java：`EnhanceInnerInterceptor extends InnerInterceptor`
/// 保留 `before_query` / `before_update` / `after_query` / `after_update`
/// + 新增 `after_execution` / `phase()` 方法。
///
/// ## 阶段声明
/// 框架内置增强必须返回明确阶段；第三方增强默认不参与强制排序。
#[async_trait]
pub trait EnhanceInnerInterceptor: Send + Sync + std::fmt::Debug {
    /// 声明增强在统一拦截器链中的阶段。
    ///
    /// 对应 Java：`EnhanceInnerInterceptor.phase()`
    /// 默认返回 `UNSPECIFIED`（不参与强制排序）。
    fn phase(&self) -> EnhancePhase {
        EnhancePhase::UNSPECIFIED
    }

    /// 查询成功完成后的结果增强处理（可修改返回对象）。
    ///
    /// 对应 Java：`EnhanceInnerInterceptor.afterQuery(...)`
    async fn after_query(
        &self,
        _executor: &dyn Executor,
        _sql: &str,
        _result: &mut Result<rbs::Value, rbatis::Error>,
    ) -> Result<(), rbatis::Error> {
        Ok(())
    }

    /// 更新成功完成后的增强处理。
    ///
    /// 对应 Java：`EnhanceInnerInterceptor.afterUpdate(...)`
    async fn after_update(
        &self,
        _executor: &dyn Executor,
        _sql: &str,
        _result: &mut Result<rbdc::db::ExecResult, rbatis::Error>,
    ) -> Result<(), rbatis::Error> {
        Ok(())
    }

    /// SQL 执行及结果增强全部完成后的生命周期通知（覆盖查询/增删改及异常路径）。
    ///
    /// 对应 Java：`EnhanceInnerInterceptor.afterExecution(...)`
    /// 实现不得抛出异常影响 SQL 主流程。
    async fn after_execution(
        &self,
        _executor: &dyn Executor,
        _sql: &str,
        _elapsed_nanos: u64,
        _failure: Option<&rbatis::Error>,
    ) {
        // 默认空操作
    }
}
