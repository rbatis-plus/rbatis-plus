// Source: mybatis-plus-extension/.../inner/InnerInterceptor.java
// Source: mybatis-plus-enhance-core/.../plugins/inner/EnhanceInnerInterceptor.java

use async_trait::async_trait;
use rbatis::executor::Executor;
use rbatis::intercept::Action;
use rbatis::{Error, plugin::transaction::TransactionEvent};
use rbdc::db::ExecResult;
use rbs::Value;

/// Inner interceptor — the core extension point for all RBatis-Plus plugins.
///
/// Mirrors Java `InnerInterceptor` (from mybatis-plus) extended with
/// `EnhanceInnerInterceptor` lifecycle hooks (from mybatis-plus-enhance).
///
/// Implementors receive before/after callbacks for query and exec operations.
/// The interceptor can modify SQL, args, or short-circuit the result.
#[async_trait]
pub trait InnerInterceptor: Send + Sync + std::fmt::Debug {
    /// Called before a query reaches the database.
    /// Return `Action::Return` to short-circuit with the result.
    async fn before_query(
        &self,
        _executor: &dyn Executor,
        _sql: &mut String,
        _args: &mut Vec<Value>,
        _result: &mut Result<Value, Error>,
    ) -> Result<Action, Error> {
        Ok(Action::Next)
    }

    /// Called after a query completes successfully.
    async fn after_query(
        &self,
        _executor: &dyn Executor,
        _sql: &str,
        _result: &mut Result<Value, Error>,
    ) -> Result<(), Error> {
        Ok(())
    }

    /// Called before an exec (INSERT/UPDATE/DELETE) reaches the database.
    async fn before_update(
        &self,
        _executor: &dyn Executor,
        _sql: &mut String,
        _args: &mut Vec<Value>,
        _result: &mut Result<ExecResult, Error>,
    ) -> Result<Action, Error> {
        Ok(Action::Next)
    }

    /// Called after an exec completes.
    async fn after_update(
        &self,
        _executor: &dyn Executor,
        _sql: &str,
        _result: &mut Result<ExecResult, Error>,
    ) -> Result<(), Error> {
        Ok(())
    }

    /// Called after any operation (query or exec) completes, in `finally`.
    /// Used for observation/metrics.
    async fn after_execution(
        &self,
        _executor: &dyn Executor,
        _sql: &str,
        _elapsed_nanos: u64,
        _failure: Option<&Error>,
    ) {
    }

    /// Called on transaction events (commit/rollback).
    async fn on_transaction_event(&self, _event: &TransactionEvent) {}
}
