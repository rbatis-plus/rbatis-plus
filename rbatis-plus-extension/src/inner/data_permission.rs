// Source: mybatis-plus-jsqlparser-5.0/.../inner/DataPermissionInterceptor.java

use crate::inner::inner_interceptor::InnerInterceptor;
use async_trait::async_trait;
use rbatis::executor::Executor;
use rbatis::{Action, Error};
use rbdc::db::ExecResult;
use rbs::Value;
use std::sync::Arc;

/// Data permission handler — provides row-level access control.
///
/// Mirrors Java `MultiDataPermissionHandler`.
pub trait DataPermissionHandler: Send + Sync + std::fmt::Debug {
    /// Given a table name and the current WHERE expression, return an
    /// additional WHERE fragment to AND-append (e.g. `"dept_id IN (1,2,3)"`).
    /// Return empty string to add no constraint.
    fn get_data_permission_where(&self, table_name: &str, current_where: &str) -> String;
}

/// Data permission inner interceptor.
///
/// Mirrors Java `DataPermissionInterceptor`.
///
/// Appends a per-table WHERE expression to enforce row-level data permissions.
#[derive(Clone)]
pub struct DataPermissionInnerInterceptor {
    pub handler: Arc<dyn DataPermissionHandler>,
}

impl std::fmt::Debug for DataPermissionInnerInterceptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataPermissionInnerInterceptor")
            .finish_non_exhaustive()
    }
}

impl DataPermissionInnerInterceptor {
    pub fn new(handler: Arc<dyn DataPermissionHandler>) -> Self {
        Self { handler }
    }

    fn append_permission(&self, sql: &mut String) {
        let fragment = self.handler.get_data_permission_where("", "");
        if fragment.is_empty() {
            return;
        }
        let lower = sql.trim_start().to_lowercase();
        if lower.contains(" where ") {
            sql.push_str(&format!(" AND ({})", fragment));
        } else {
            sql.push_str(&format!(" WHERE ({})", fragment));
        }
    }
}

#[async_trait]
impl InnerInterceptor for DataPermissionInnerInterceptor {
    async fn before_query(
        &self,
        _executor: &dyn Executor,
        sql: &mut String,
        _args: &mut Vec<Value>,
        _result: &mut Result<Value, Error>,
    ) -> Result<Action, Error> {
        self.append_permission(sql);
        Ok(Action::Next)
    }

    async fn before_update(
        &self,
        _executor: &dyn Executor,
        sql: &mut String,
        _args: &mut Vec<Value>,
        _result: &mut Result<ExecResult, Error>,
    ) -> Result<Action, Error> {
        self.append_permission(sql);
        Ok(Action::Next)
    }
}
