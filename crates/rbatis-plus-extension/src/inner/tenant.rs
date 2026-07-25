// Source: mybatis-plus-jsqlparser-5.0/.../inner/TenantLineInnerInterceptor.java
// Source: mybatis-plus-enhance-extension/.../tenant/DefaultTenantLineHandler.java

use crate::inner::inner_interceptor::InnerInterceptor;
use async_trait::async_trait;
use rbatis::executor::Executor;
use rbatis::{Action, Error};
use rbdc::db::ExecResult;
use rbs::Value;
use std::sync::Arc;

/// Tenant line handler — provides tenant ID and configuration.
///
/// Mirrors Java `TenantLineHandler`.
pub trait TenantLineHandler: Send + Sync + std::fmt::Debug {
    /// Get the current tenant ID value.
    fn get_tenant_id(&self) -> Value;

    /// Get the tenant column name.
    fn get_tenant_id_column(&self) -> &str {
        "tenant_id"
    }

    /// Whether to ignore the given table for tenant filtering.
    fn ignore_table(&self, _table_name: &str) -> bool {
        false
    }
}

/// Tenant inner interceptor.
///
/// Mirrors Java `TenantLineInnerInterceptor`.
///
/// Appends `AND tenant_id = ?` to SELECT/UPDATE/DELETE queries
/// and adds the tenant column to INSERT values.
#[derive(Clone)]
pub struct TenantInnerInterceptor {
    pub handler: Arc<dyn TenantLineHandler>,
}

impl std::fmt::Debug for TenantInnerInterceptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TenantInnerInterceptor").finish_non_exhaustive()
    }
}

impl TenantInnerInterceptor {
    pub fn new(handler: Arc<dyn TenantLineHandler>) -> Self {
        Self { handler }
    }

    fn append_tenant_condition(&self, sql: &mut String) {
        let column = self.handler.get_tenant_id_column();
        let tenant_id = self.handler.get_tenant_id();

        let lower = sql.trim_start().to_lowercase();
        if lower.contains(" where ") {
            sql.push_str(&format!(" AND {} = {}", column, Self::value_literal(&tenant_id)));
        } else {
            sql.push_str(&format!(" WHERE {} = {}", column, Self::value_literal(&tenant_id)));
        }
    }

    fn value_literal(v: &Value) -> String {
        match v {
            Value::Null => "NULL".to_string(),
            Value::String(s) => format!("'{}'", s.replace("'", "''")),
            _ => v.to_string(),
        }
    }
}

#[async_trait]
impl InnerInterceptor for TenantInnerInterceptor {
    async fn before_query(
        &self,
        _executor: &dyn Executor,
        sql: &mut String,
        _args: &mut Vec<Value>,
        _result: &mut Result<Value, Error>,
    ) -> Result<Action, Error> {
        self.append_tenant_condition(sql);
        Ok(Action::Next)
    }

    async fn before_update(
        &self,
        _executor: &dyn Executor,
        sql: &mut String,
        _args: &mut Vec<Value>,
        _result: &mut Result<ExecResult, Error>,
    ) -> Result<Action, Error> {
        self.append_tenant_condition(sql);
        Ok(Action::Next)
    }
}
