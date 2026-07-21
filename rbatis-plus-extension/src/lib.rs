#![forbid(unsafe_code)]
//! Built-in interceptors aligned with the fixed rbatis-plus stage model.

use futures::future::BoxFuture;
use rbatis_plus_core::{InterceptorStage, PlusResult, SqlInterceptor, SqlInvocation};
use serde_json::Value;
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};

mod audit;
mod mapper;
mod security;
pub use audit::{AuditContext, AuditFields, AuditFillInterceptor, AuditOperation};
pub use mapper::RbatisMapper;
pub use security::{
    AesGcmKeyRing, EncryptedParameter, FieldCipher, FieldDecryptionInterceptor,
    FieldEncryptionInterceptor, PartialRowPolicy, RowSignature, RowSignatureService,
    RowSignatureVerificationInterceptor, SignatureScope, VerificationOutcome,
};

pub trait DataScopeProvider: Send + Sync {
    fn condition(&self, statement_id: &str) -> Option<String>;
}

pub struct DataPermissionInterceptor<P> {
    provider: P,
}
impl<P> DataPermissionInterceptor<P> {
    pub fn new(provider: P) -> Self {
        Self { provider }
    }
}
impl<P: DataScopeProvider> SqlInterceptor for DataPermissionInterceptor<P> {
    fn stage(&self) -> InterceptorStage {
        InterceptorStage::SqlRewrite
    }
    fn intercept<'a>(&'a self, invocation: &'a mut SqlInvocation) -> BoxFuture<'a, PlusResult<()>> {
        Box::pin(async move {
            if let Some(condition) = self
                .provider
                .condition(&invocation.statement_id)
                .filter(|value| !value.trim().is_empty())
            {
                let connector = if invocation.sql.to_ascii_lowercase().contains(" where ") {
                    " AND "
                } else {
                    " WHERE "
                };
                invocation.sql.push_str(connector);
                invocation.sql.push('(');
                invocation.sql.push_str(&condition);
                invocation.sql.push(')');
            }
            Ok(())
        })
    }
}

pub struct TenantInterceptor {
    tenant_id: String,
    column: String,
}
impl TenantInterceptor {
    pub fn new(tenant_id: impl Into<String>, column: impl Into<String>) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            column: column.into(),
        }
    }
}
impl SqlInterceptor for TenantInterceptor {
    fn stage(&self) -> InterceptorStage {
        InterceptorStage::SqlRewrite
    }
    fn intercept<'a>(&'a self, invocation: &'a mut SqlInvocation) -> BoxFuture<'a, PlusResult<()>> {
        Box::pin(async move {
            let connector = if invocation.sql.to_ascii_lowercase().contains(" where ") {
                " AND "
            } else {
                " WHERE "
            };
            invocation.sql.push_str(connector);
            let _ = write!(invocation.sql, "{} = ?", self.column);
            invocation
                .parameters
                .push(Value::String(self.tenant_id.clone()));
            Ok(())
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlObservation {
    pub statement_id: String,
    pub normalized_sql: String,
}

#[derive(Default)]
pub struct ObservationInterceptor {
    observations: Arc<Mutex<Vec<SqlObservation>>>,
}
impl ObservationInterceptor {
    pub fn observations(&self) -> Arc<Mutex<Vec<SqlObservation>>> {
        self.observations.clone()
    }
}
impl SqlInterceptor for ObservationInterceptor {
    fn stage(&self) -> InterceptorStage {
        InterceptorStage::Observe
    }
    fn intercept<'a>(&'a self, invocation: &'a mut SqlInvocation) -> BoxFuture<'a, PlusResult<()>> {
        Box::pin(async move {
            let normalized_sql = invocation
                .sql
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            self.observations
                .lock()
                .expect("observation lock poisoned")
                .push(SqlObservation {
                    statement_id: invocation.statement_id.clone(),
                    normalized_sql,
                });
            Ok(())
        })
    }
}
