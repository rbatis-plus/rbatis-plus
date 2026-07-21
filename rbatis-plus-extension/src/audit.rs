use futures::future::BoxFuture;
use rbatis_plus_core::{InterceptorStage, PlusError, PlusResult, SqlInterceptor, SqlInvocation};
use serde_json::{Map, Value};

/// Request-scoped values used by insert/update audit filling.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditContext {
    pub now: Value,
    pub tenant_id: Option<Value>,
    pub system_id: Option<Value>,
    pub actor: Option<Value>,
}

/// Persistence-field names generated from model metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditFields {
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub tenant_id: Option<String>,
    pub system_id: Option<String>,
    pub logic_delete: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditOperation {
    Insert,
    Update,
}

/// Fills null or absent audit fields during the parameter-transform stage.
pub struct AuditFillInterceptor {
    context: AuditContext,
    fields: AuditFields,
    operation: AuditOperation,
    parameter_indices: Vec<usize>,
}

impl AuditFillInterceptor {
    pub fn new(
        context: AuditContext,
        fields: AuditFields,
        operation: AuditOperation,
        parameter_indices: Vec<usize>,
    ) -> Self {
        Self {
            context,
            fields,
            operation,
            parameter_indices,
        }
    }
}

impl SqlInterceptor for AuditFillInterceptor {
    fn stage(&self) -> InterceptorStage {
        InterceptorStage::ParameterTransform
    }

    fn intercept<'a>(&'a self, invocation: &'a mut SqlInvocation) -> BoxFuture<'a, PlusResult<()>> {
        Box::pin(async move {
            for index in &self.parameter_indices {
                let parameter = invocation.parameters.get_mut(*index).ok_or_else(|| {
                    PlusError::InvalidArgument(format!(
                        "audit parameter index {index} is out of bounds"
                    ))
                })?;
                let row = parameter.as_object_mut().ok_or_else(|| {
                    PlusError::InvalidArgument(format!("audit parameter {index} must be an object"))
                })?;
                match self.operation {
                    AuditOperation::Insert => fill_insert(row, &self.fields, &self.context),
                    AuditOperation::Update => fill_update(row, &self.fields, &self.context),
                }
            }
            Ok(())
        })
    }
}

fn fill_insert(row: &mut Map<String, Value>, fields: &AuditFields, context: &AuditContext) {
    fill_empty(row, fields.created_at.as_deref(), Some(&context.now));
    fill_empty(row, fields.updated_at.as_deref(), Some(&context.now));
    fill_empty(row, fields.created_by.as_deref(), context.actor.as_ref());
    fill_empty(row, fields.updated_by.as_deref(), context.actor.as_ref());
    fill_empty(row, fields.tenant_id.as_deref(), context.tenant_id.as_ref());
    fill_empty(row, fields.system_id.as_deref(), context.system_id.as_ref());
    let logic_zero = Value::from(0);
    fill_empty(row, fields.logic_delete.as_deref(), Some(&logic_zero));
}

fn fill_update(row: &mut Map<String, Value>, fields: &AuditFields, context: &AuditContext) {
    fill_empty(row, fields.updated_at.as_deref(), Some(&context.now));
    fill_empty(row, fields.updated_by.as_deref(), context.actor.as_ref());
}

fn fill_empty(row: &mut Map<String, Value>, field: Option<&str>, value: Option<&Value>) {
    let (Some(field), Some(value)) = (field, value) else {
        return;
    };
    if row.get(field).is_none_or(Value::is_null) {
        row.insert(field.to_owned(), value.clone());
    }
}
