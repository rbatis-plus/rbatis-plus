use rbatis_plus_core::{InterceptorChain, InterceptorStage, SqlInvocation};
use rbatis_plus_extension::{
    AuditContext, AuditFields, AuditFillInterceptor, AuditOperation, DataPermissionInterceptor,
    DataScopeProvider, ObservationInterceptor, TenantInterceptor,
};
use serde_json::{Value, json};
use std::sync::Arc;

struct Scope;
impl DataScopeProvider for Scope {
    fn condition(&self, _: &str) -> Option<String> {
        Some("department_id IN (10, 20)".into())
    }
}

#[tokio::test]
async fn audit_fill_uses_context_without_overwriting_explicit_values() {
    let fields = AuditFields {
        created_at: Some("created_at".to_owned()),
        updated_at: Some("updated_at".to_owned()),
        created_by: Some("created_by".to_owned()),
        updated_by: Some("updated_by".to_owned()),
        tenant_id: Some("tenant_id".to_owned()),
        system_id: Some("system_id".to_owned()),
        logic_delete: Some("deleted".to_owned()),
    };
    let context = AuditContext {
        now: Value::String("2026-07-21T12:00:00Z".to_owned()),
        tenant_id: Some(Value::String("tenant-a".to_owned())),
        system_id: Some(Value::String("ddd4r".to_owned())),
        actor: Some(Value::String("user-1".to_owned())),
    };
    let chain = InterceptorChain::new(vec![Arc::new(AuditFillInterceptor::new(
        context,
        fields,
        AuditOperation::Insert,
        vec![0],
    ))]);
    let mut invocation = SqlInvocation::new(
        "OrderMapper.insert",
        "INSERT",
        vec![json!({"id": 1, "created_by": "importer"})],
    );
    chain.apply(&mut invocation).await.unwrap();
    let row = invocation.parameters[0].as_object().unwrap();
    assert_eq!(row["created_by"], "importer");
    assert_eq!(row["updated_by"], "user-1");
    assert_eq!(row["tenant_id"], "tenant-a");
    assert_eq!(row["system_id"], "ddd4r");
    assert_eq!(row["deleted"], 0);
    assert_eq!(row["created_at"], "2026-07-21T12:00:00Z");
}

#[tokio::test]
async fn orders_rewrite_before_observation_and_combines_tenant_scope() {
    let observation = Arc::new(ObservationInterceptor::default());
    let records = observation.observations();
    let chain = InterceptorChain::new(vec![
        observation,
        Arc::new(TenantInterceptor::new("tenant-a", "tenant_id")),
        Arc::new(DataPermissionInterceptor::new(Scope)),
    ]);
    assert_eq!(
        chain.stages(),
        vec![
            InterceptorStage::SqlRewrite,
            InterceptorStage::SqlRewrite,
            InterceptorStage::Observe
        ]
    );
    let mut invocation =
        SqlInvocation::new("OrderMapper.selectList", "SELECT * FROM orders", vec![]);
    chain.apply(&mut invocation).await.unwrap();
    assert!(invocation.sql.contains("department_id IN (10, 20)"));
    assert!(invocation.sql.contains("tenant_id = ?"));
    assert_eq!(invocation.parameters.len(), 1);
    assert_eq!(records.lock().unwrap()[0].normalized_sql, invocation.sql);
}
