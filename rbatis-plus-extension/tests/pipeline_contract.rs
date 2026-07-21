use rbatis_plus_core::{InterceptorChain, InterceptorStage, SqlInvocation};
use rbatis_plus_extension::{
    DataPermissionInterceptor, DataScopeProvider, ObservationInterceptor, TenantInterceptor,
};
use std::sync::Arc;

struct Scope;
impl DataScopeProvider for Scope {
    fn condition(&self, _: &str) -> Option<String> {
        Some("department_id IN (10, 20)".into())
    }
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
