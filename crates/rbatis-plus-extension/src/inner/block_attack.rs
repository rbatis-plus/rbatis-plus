// Source: mybatis-plus-jsqlparser-5.0/.../inner/BlockAttackInnerInterceptor.java

use crate::inner::inner_interceptor::InnerInterceptor;
use async_trait::async_trait;
use rbatis::executor::Executor;
use rbatis::{Action, Error};
use rbdc::db::ExecResult;
use rbs::Value;

/// Block-attack inner interceptor.
///
/// Mirrors Java `BlockAttackInnerInterceptor`.
///
/// Prevents full-table UPDATE or DELETE operations (no WHERE clause).
/// Throws an error if an UPDATE/DELETE SQL has no WHERE condition.
#[derive(Debug, Clone, Default)]
pub struct BlockAttackInnerInterceptor;

impl BlockAttackInnerInterceptor {
    pub fn new() -> Self {
        Self
    }

    fn has_where(sql: &str) -> bool {
        let lower = sql.trim_start().to_lowercase();
        lower.contains(" where ")
    }
}

#[async_trait]
impl InnerInterceptor for BlockAttackInnerInterceptor {
    async fn before_update(
        &self,
        _executor: &dyn Executor,
        sql: &mut String,
        _args: &mut Vec<Value>,
        _result: &mut Result<ExecResult, Error>,
    ) -> Result<Action, Error> {
        let lower = sql.trim_start().to_lowercase();
        let is_update_or_delete =
            lower.starts_with("update ") || lower.starts_with("delete from");

        if is_update_or_delete && !Self::has_where(sql) {
            return Err(Error::from(
                "[rbatis-plus] Prohibition of full table update/delete operation",
            ));
        }
        Ok(Action::Next)
    }
}
