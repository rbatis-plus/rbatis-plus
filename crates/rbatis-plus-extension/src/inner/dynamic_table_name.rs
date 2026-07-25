// Source: mybatis-plus-jsqlparser-5.0/.../inner/DynamicTableNameInnerInterceptor.java

use crate::inner::inner_interceptor::InnerInterceptor;
use async_trait::async_trait;
use rbatis::executor::Executor;
use rbatis::{Action, Error};
use rbdc::db::ExecResult;
use rbs::Value;
use std::sync::Arc;

/// Table name handler — provides runtime table-name resolution.
///
/// Mirrors Java `TableNameHandler`.
pub trait TableNameHandler: Send + Sync + std::fmt::Debug {
    /// Given the original SQL and the current table name, return the
    /// replacement table name.
    fn dynamic_table_name(&self, sql: &str, table_name: &str) -> String;
}

/// Dynamic table name inner interceptor.
///
/// Mirrors Java `DynamicTableNameInnerInterceptor`.
///
/// Rewrites table names in SQL at runtime — useful for date-sharded tables,
/// tenant-by-table routing, etc.
#[derive(Clone)]
pub struct DynamicTableNameInnerInterceptor {
    pub handler: Arc<dyn TableNameHandler>,
}

impl std::fmt::Debug for DynamicTableNameInnerInterceptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynamicTableNameInnerInterceptor")
            .finish_non_exhaustive()
    }
}

impl DynamicTableNameInnerInterceptor {
    pub fn new(handler: Arc<dyn TableNameHandler>) -> Self {
        Self { handler }
    }
}

#[async_trait]
impl InnerInterceptor for DynamicTableNameInnerInterceptor {
    async fn before_query(
        &self,
        _executor: &dyn Executor,
        sql: &mut String,
        _args: &mut Vec<Value>,
        _result: &mut Result<Value, Error>,
    ) -> Result<Action, Error> {
        // In a full implementation this would parse SQL and replace table tokens.
        // For MVP, the handler is called with the full SQL string.
        let original = sql.clone();
        *sql = self.handler.dynamic_table_name(&original, "");
        Ok(Action::Next)
    }

    async fn before_update(
        &self,
        _executor: &dyn Executor,
        sql: &mut String,
        _args: &mut Vec<Value>,
        _result: &mut Result<ExecResult, Error>,
    ) -> Result<Action, Error> {
        let original = sql.clone();
        *sql = self.handler.dynamic_table_name(&original, "");
        Ok(Action::Next)
    }
}
