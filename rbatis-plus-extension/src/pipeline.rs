use crate::security::{
    EncryptedParameter, FieldCipher, FieldDecryptionInterceptor, FieldEncryptionInterceptor,
    PartialRowPolicy, RowSignatureService, RowSignatureVerificationInterceptor, SignatureScope,
};
use rbatis_plus_core::{InterceptorChain, InterceptorStage, PlusError, PlusResult, SqlInterceptor};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Required row-signature verification policy for a secure mapper pipeline.
pub struct RowVerificationConfig {
    service: Arc<RowSignatureService>,
    expected_columns: Vec<String>,
    signed_columns: Vec<String>,
    scope: SignatureScope,
    partial_policy: PartialRowPolicy,
    key_id_field: String,
    signature_field: String,
}

impl RowVerificationConfig {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        service: Arc<RowSignatureService>,
        expected_columns: Vec<String>,
        signed_columns: Vec<String>,
        scope: SignatureScope,
        partial_policy: PartialRowPolicy,
        key_id_field: impl Into<String>,
        signature_field: impl Into<String>,
    ) -> PlusResult<Self> {
        let key_id_field = key_id_field.into();
        let signature_field = signature_field.into();
        if expected_columns.is_empty() {
            return Err(PlusError::InvalidArgument(
                "secure pipeline requires expected signed columns".to_owned(),
            ));
        }
        if key_id_field.trim().is_empty() || signature_field.trim().is_empty() {
            return Err(PlusError::InvalidArgument(
                "signature metadata fields must not be empty".to_owned(),
            ));
        }
        Ok(Self {
            service,
            expected_columns,
            signed_columns,
            scope,
            partial_policy,
            key_id_field,
            signature_field,
        })
    }
}

struct EncryptionRule {
    statement_ids: Vec<String>,
    parameters: Vec<EncryptedParameter>,
}

/// Builds a fail-closed chain with security-sensitive stages in a fixed order.
///
/// Callers may add SQL rewrite and observation interceptors, but cannot replace
/// parameter encryption, row verification, or result decryption stages.
pub struct SecurePipelineBuilder<C> {
    cipher: Arc<C>,
    verification: RowVerificationConfig,
    encryption_rules: Vec<EncryptionRule>,
    decrypted_fields: BTreeMap<String, Vec<u8>>,
    extensions: Vec<Arc<dyn SqlInterceptor>>,
}

impl<C: FieldCipher + 'static> SecurePipelineBuilder<C> {
    pub fn new(cipher: Arc<C>, verification: RowVerificationConfig) -> Self {
        Self {
            cipher,
            verification,
            encryption_rules: Vec::new(),
            decrypted_fields: BTreeMap::new(),
            extensions: Vec::new(),
        }
    }

    #[must_use]
    pub fn encrypt_parameters_for(
        mut self,
        statement_ids: impl IntoIterator<Item = impl Into<String>>,
        parameters: Vec<EncryptedParameter>,
    ) -> Self {
        self.encryption_rules.push(EncryptionRule {
            statement_ids: statement_ids.into_iter().map(Into::into).collect(),
            parameters,
        });
        self
    }

    #[must_use]
    pub fn decrypt_fields(mut self, fields: BTreeMap<String, Vec<u8>>) -> Self {
        self.decrypted_fields = fields;
        self
    }

    #[must_use]
    pub fn with_interceptor(mut self, interceptor: Arc<dyn SqlInterceptor>) -> Self {
        self.extensions.push(interceptor);
        self
    }

    pub fn build(self) -> PlusResult<Arc<InterceptorChain>> {
        if self.decrypted_fields.is_empty() {
            return Err(PlusError::InvalidArgument(
                "secure pipeline requires at least one encrypted result field".to_owned(),
            ));
        }
        if self
            .encryption_rules
            .iter()
            .any(|rule| rule.statement_ids.is_empty() || rule.parameters.is_empty())
        {
            return Err(PlusError::InvalidArgument(
                "encryption rules require statement IDs and parameters".to_owned(),
            ));
        }
        if let Some(stage) = self
            .extensions
            .iter()
            .map(|item| item.stage())
            .find(|stage| {
                !matches!(
                    stage,
                    InterceptorStage::SqlRewrite | InterceptorStage::Observe
                )
            })
        {
            return Err(PlusError::InvalidArgument(format!(
                "secure pipeline extension cannot override reserved stage {stage:?}"
            )));
        }

        let RowVerificationConfig {
            service,
            expected_columns,
            signed_columns,
            scope,
            partial_policy,
            key_id_field,
            signature_field,
        } = self.verification;
        let mut interceptors = self.extensions;
        for rule in self.encryption_rules {
            interceptors.push(Arc::new(
                FieldEncryptionInterceptor::new(self.cipher.clone(), rule.parameters)
                    .for_statements(rule.statement_ids),
            ));
        }
        interceptors.push(Arc::new(RowSignatureVerificationInterceptor::new(
            service,
            expected_columns,
            signed_columns,
            scope,
            partial_policy,
            key_id_field,
            signature_field,
        )));
        interceptors.push(Arc::new(FieldDecryptionInterceptor::new(
            self.cipher,
            self.decrypted_fields,
        )));
        Ok(Arc::new(InterceptorChain::new(interceptors)))
    }
}
