use crate::{CacheError, CacheKey, CachePolicy, CacheStore};
use async_trait::async_trait;
use rbatis::executor::Executor;
use rbatis::intercept::{Intercept, ResultType};
use rbatis::plugin::lifecycle::{infer_executor_type, ExecutorType};
use rbatis::{Action, Error};
use rbdc::db::ExecResult;
use rbs::Value;
use std::sync::Arc;

/// The main cache interceptor.  Plug this into `RBatis`'s intercept chain
/// **after** SQL-rewriting interceptors (Page, Tenant, ...) and **before**
/// LogInterceptor.
///
/// Behaviour:
/// - **Query before**: build key, look up cache.  On hit, write the cached
///   `Value` into the result slot and return `Action::Return` to short-circuit
///   the database call.
/// - **Query after**: if the database returned a result, store it in the cache.
/// - **Exec (DML) after**: if rows_affected > 0, invalidate the namespace.
/// - **Transaction queries**: bypassed (returns `Action::Next` without
///   touching the cache).
/// - **Errors**: fail-open (logged, not propagated).
pub struct CacheIntercept {
    pub store: Arc<dyn CacheStore>,
    pub policy: CachePolicy,
}

impl std::fmt::Debug for CacheIntercept {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheIntercept")
            .field("namespace", &self.policy.namespace)
            .finish_non_exhaustive()
    }
}

impl Clone for CacheIntercept {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            policy: self.policy.clone(),
        }
    }
}

impl CacheIntercept {
    pub fn new(store: Arc<dyn CacheStore>, policy: CachePolicy) -> Self {
        Self { store, policy }
    }

    fn build_key(&self, sql: &str, args: &[Value]) -> CacheKey {
        CacheKey::new(&self.policy.namespace, sql, args.to_vec())
    }

    fn should_bypass(&self, executor: &dyn Executor) -> bool {
        // Bypass transaction executors entirely.
        let exec_type = infer_executor_type(executor);
        matches!(exec_type, ExecutorType::Transaction | ExecutorType::TransactionGuard)
    }

    fn log_cache_error(&self, context: &str, e: CacheError) {
        log::warn!("rbatis-plus cache {} (fail-open): {}", context, e);
    }

    /// Check whether a SQL string is a SELECT (cacheable) vs DML (invalidation).
    fn is_select(sql: &str) -> bool {
        let trimmed = sql.trim_start().to_lowercase();
        trimmed.starts_with("select ") || trimmed.starts_with("select\n")
    }
}

#[async_trait]
impl Intercept for CacheIntercept {
    async fn before(
        &self,
        _task_id: i64,
        rb: &dyn Executor,
        sql: &mut String,
        args: &mut Vec<Value>,
        result: ResultType<&mut Result<ExecResult, Error>, &mut Result<Value, Error>>,
    ) -> Result<Action, Error> {
        // Only handle Query operations
        let ResultType::Query(query_result) = result else {
            return Ok(Action::Next);
        };

        // Bypass transactions
        if self.should_bypass(rb) {
            return Ok(Action::Next);
        }

        // Only cache SELECTs
        if !Self::is_select(sql) {
            return Ok(Action::Next);
        }

        let key = self.build_key(sql, args);

        // Look up cache
        match self.store.get(&key).await {
            Ok(Some(cached_value)) => {
                log::debug!("rbatis-plus cache HIT: {:?}", key);
                *query_result = Ok(cached_value);
                Ok(Action::Return) // Short-circuit: skip database
            }
            Ok(None) => {
                log::debug!("rbatis-plus cache MISS: {:?}", key);
                Ok(Action::Next)
            }
            Err(e) => {
                self.log_cache_error("get (fail-open)", e);
                Ok(Action::Next)
            }
        }
    }

    async fn after(
        &self,
        _task_id: i64,
        rb: &dyn Executor,
        sql: &mut String,
        args: &mut Vec<Value>,
        result: ResultType<&mut Result<ExecResult, Error>, &mut Result<Value, Error>>,
    ) -> Result<Action, Error> {
        // Bypass transactions
        if self.should_bypass(rb) {
            return Ok(Action::Next);
        }

        match result {
            ResultType::Query(query_result) => {
                // Only cache SELECTs
                if !Self::is_select(sql) {
                    return Ok(Action::Next);
                }

                // Only cache successful results
                let Ok(value) = query_result.as_ref() else {
                    return Ok(Action::Next);
                };

                // Check empty result policy
                if !self.policy.cache_null {
                    if let Value::Array(arr) = value {
                        if arr.is_empty() {
                            return Ok(Action::Next);
                        }
                    }
                }

                // Store in cache
                let key = self.build_key(sql, args);
                let ttl = if is_empty_result(value) {
                    self.policy.null_ttl.unwrap_or(self.policy.ttl)
                } else {
                    self.policy.ttl
                };

                if let Err(e) = self
                    .store
                    .set(key, value.clone(), ttl, &[self.policy.namespace.clone()])
                    .await
                {
                    self.log_cache_error("set (fail-open)", e);
                }
                Ok(Action::Next)
            }
            ResultType::Exec(exec_result) => {
                // DML invalidation: if rows_affected > 0, clear namespace
                let Ok(er) = exec_result.as_ref() else {
                    return Ok(Action::Next);
                };

                if er.rows_affected > 0 {
                    log::debug!(
                        "rbatis-plus cache invalidating namespace '{}' (DML affected {} rows)",
                        self.policy.namespace,
                        er.rows_affected
                    );
                    if let Err(e) = self.store.clear_namespace(&self.policy.namespace).await {
                        self.log_cache_error("clear_namespace (fail-open)", e);
                    }
                }
                Ok(Action::Next)
            }
        }
    }
}

fn is_empty_result(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::Array(arr) => arr.is_empty(),
        _ => false,
    }
}
