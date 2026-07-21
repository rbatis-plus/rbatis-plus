use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, Generate, KeyInit, Payload},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use futures::future::BoxFuture;
use hmac::{Hmac, Mac};
use rbatis_plus_core::{InterceptorStage, PlusError, PlusResult, SqlInterceptor, SqlInvocation};
use serde_json::{Map, Value};
use sha2::Sha256;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use zeroize::Zeroizing;

type HmacSha256 = Hmac<Sha256>;

/// Encryption and deterministic blind-index contract for persistence fields.
pub trait FieldCipher: Send + Sync {
    fn encrypt(&self, plaintext: &[u8], context: &[u8]) -> PlusResult<String>;
    fn decrypt(&self, envelope: &str, context: &[u8]) -> PlusResult<Vec<u8>>;
    fn blind_index(&self, plaintext: &[u8], context: &[u8]) -> PlusResult<String>;
}

/// Versioned AES-256-GCM key ring with an independently keyed blind index.
///
/// Ciphertexts encode `version.key-id.nonce.ciphertext-and-tag`; decryption
/// selects the embedded key, which permits online key rotation.
pub struct AesGcmKeyRing {
    active_key_id: String,
    keys: BTreeMap<String, Zeroizing<[u8; 32]>>,
    blind_index_key: Zeroizing<[u8; 32]>,
}

impl AesGcmKeyRing {
    pub fn new(
        active_key_id: impl Into<String>,
        keys: impl IntoIterator<Item = (String, [u8; 32])>,
        blind_index_key: [u8; 32],
    ) -> PlusResult<Self> {
        let active_key_id = active_key_id.into();
        if active_key_id.is_empty() || active_key_id.contains('.') {
            return Err(PlusError::InvalidArgument(
                "active key id must be non-empty and must not contain `.`".to_owned(),
            ));
        }
        let keys = keys
            .into_iter()
            .map(|(id, key)| (id, Zeroizing::new(key)))
            .collect::<BTreeMap<_, _>>();
        if !keys.contains_key(&active_key_id) {
            return Err(PlusError::InvalidArgument(format!(
                "active encryption key `{active_key_id}` is missing"
            )));
        }
        if keys.keys().any(|id| id.is_empty() || id.contains('.')) {
            return Err(PlusError::InvalidArgument(
                "encryption key ids must be non-empty and must not contain `.`".to_owned(),
            ));
        }
        Ok(Self {
            active_key_id,
            keys,
            blind_index_key: Zeroizing::new(blind_index_key),
        })
    }

    pub fn active_key_id(&self) -> &str {
        &self.active_key_id
    }
}

impl FieldCipher for AesGcmKeyRing {
    fn encrypt(&self, plaintext: &[u8], context: &[u8]) -> PlusResult<String> {
        let key = self
            .keys
            .get(&self.active_key_id)
            .expect("active key validated at construction");
        let cipher = Aes256Gcm::new_from_slice(key.as_ref()).map_err(crypto_error)?;
        let nonce = Nonce::generate();
        let ciphertext = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: plaintext,
                    aad: context,
                },
            )
            .map_err(crypto_error)?;
        Ok(format!(
            "v1.{}.{}.{}",
            self.active_key_id,
            URL_SAFE_NO_PAD.encode(nonce),
            URL_SAFE_NO_PAD.encode(ciphertext)
        ))
    }

    fn decrypt(&self, envelope: &str, context: &[u8]) -> PlusResult<Vec<u8>> {
        let mut parts = envelope.split('.');
        let (Some("v1"), Some(key_id), Some(nonce), Some(ciphertext), None) = (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ) else {
            return Err(PlusError::InvalidArgument(
                "invalid encrypted field envelope".to_owned(),
            ));
        };
        let key = self.keys.get(key_id).ok_or_else(|| {
            PlusError::InvalidArgument(format!("unknown encryption key `{key_id}`"))
        })?;
        let nonce = URL_SAFE_NO_PAD.decode(nonce).map_err(crypto_error)?;
        let nonce: [u8; 12] = nonce.try_into().map_err(|value: Vec<u8>| {
            PlusError::InvalidArgument(format!("invalid AES-GCM nonce length: {}", value.len()))
        })?;
        let ciphertext = URL_SAFE_NO_PAD.decode(ciphertext).map_err(crypto_error)?;
        let cipher = Aes256Gcm::new_from_slice(key.as_ref()).map_err(crypto_error)?;
        cipher
            .decrypt(
                &nonce.into(),
                Payload {
                    msg: &ciphertext,
                    aad: context,
                },
            )
            .map_err(|_| PlusError::Interceptor {
                stage: InterceptorStage::ResultVerify,
                message: "encrypted field authentication failed".to_owned(),
            })
    }

    fn blind_index(&self, plaintext: &[u8], context: &[u8]) -> PlusResult<String> {
        let mut mac = <HmacSha256 as Mac>::new_from_slice(self.blind_index_key.as_ref())
            .map_err(crypto_error)?;
        mac.update(context);
        mac.update(&[0]);
        mac.update(plaintext);
        Ok(URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptedParameter {
    pub index: usize,
    pub context: Vec<u8>,
}

/// Encrypts configured string parameters before database execution.
pub struct FieldEncryptionInterceptor<C> {
    cipher: Arc<C>,
    parameters: Vec<EncryptedParameter>,
}

impl<C> FieldEncryptionInterceptor<C> {
    pub fn new(cipher: Arc<C>, parameters: Vec<EncryptedParameter>) -> Self {
        Self { cipher, parameters }
    }
}

impl<C: FieldCipher + 'static> SqlInterceptor for FieldEncryptionInterceptor<C> {
    fn stage(&self) -> InterceptorStage {
        InterceptorStage::ParameterTransform
    }

    fn intercept<'a>(&'a self, invocation: &'a mut SqlInvocation) -> BoxFuture<'a, PlusResult<()>> {
        Box::pin(async move {
            for encrypted in &self.parameters {
                let parameter =
                    invocation
                        .parameters
                        .get_mut(encrypted.index)
                        .ok_or_else(|| {
                            PlusError::InvalidArgument(format!(
                                "encrypted parameter index {} is out of bounds",
                                encrypted.index
                            ))
                        })?;
                let Value::String(plaintext) = parameter else {
                    return Err(PlusError::InvalidArgument(format!(
                        "encrypted parameter {} must be a string",
                        encrypted.index
                    )));
                };
                *parameter = Value::String(
                    self.cipher
                        .encrypt(plaintext.as_bytes(), &encrypted.context)?,
                );
            }
            Ok(())
        })
    }
}

/// Decrypts configured fields after result verification and before mapping.
pub struct FieldDecryptionInterceptor<C> {
    cipher: Arc<C>,
    fields: BTreeMap<String, Vec<u8>>,
}

impl<C> FieldDecryptionInterceptor<C> {
    pub fn new(cipher: Arc<C>, fields: BTreeMap<String, Vec<u8>>) -> Self {
        Self { cipher, fields }
    }
}

impl<C: FieldCipher + 'static> SqlInterceptor for FieldDecryptionInterceptor<C> {
    fn stage(&self) -> InterceptorStage {
        InterceptorStage::ResultTransform
    }

    fn intercept<'a>(&'a self, invocation: &'a mut SqlInvocation) -> BoxFuture<'a, PlusResult<()>> {
        Box::pin(async move {
            if let Some(result) = invocation.result.as_mut() {
                decrypt_result(self.cipher.as_ref(), &self.fields, result)?;
            }
            Ok(())
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartialRowPolicy {
    RejectPartial,
    DeferredResign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureScope {
    FullRow,
    SignatureOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowSignature {
    pub key_id: String,
    pub digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationOutcome {
    Verified,
    VerifiedNeedsResign,
    DeferredResign,
}

/// Canonical HMAC row signatures with rotation and partial-row policies.
pub struct RowSignatureService {
    active_key_id: String,
    keys: BTreeMap<String, Zeroizing<Vec<u8>>>,
}

/// Verifies signed database rows before any result transformation or decryption.
pub struct RowSignatureVerificationInterceptor {
    service: Arc<RowSignatureService>,
    expected_columns: Vec<String>,
    signed_columns: Vec<String>,
    scope: SignatureScope,
    partial_policy: PartialRowPolicy,
    key_id_field: String,
    signature_field: String,
}

impl RowSignatureVerificationInterceptor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        service: Arc<RowSignatureService>,
        expected_columns: Vec<String>,
        signed_columns: Vec<String>,
        scope: SignatureScope,
        partial_policy: PartialRowPolicy,
        key_id_field: impl Into<String>,
        signature_field: impl Into<String>,
    ) -> Self {
        Self {
            service,
            expected_columns,
            signed_columns,
            scope,
            partial_policy,
            key_id_field: key_id_field.into(),
            signature_field: signature_field.into(),
        }
    }
}

impl SqlInterceptor for RowSignatureVerificationInterceptor {
    fn stage(&self) -> InterceptorStage {
        InterceptorStage::ResultVerify
    }

    fn intercept<'a>(&'a self, invocation: &'a mut SqlInvocation) -> BoxFuture<'a, PlusResult<()>> {
        Box::pin(async move {
            let Some(result) = invocation.result.as_ref() else {
                return Ok(());
            };
            let expected = self
                .expected_columns
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>();
            let signed = self
                .signed_columns
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>();
            let rows = match result {
                Value::Array(rows) => rows.iter().collect::<Vec<_>>(),
                Value::Object(_) => vec![result],
                _ => {
                    return Err(PlusError::InvalidArgument(
                        "signed result must be a row or row array".to_owned(),
                    ));
                }
            };
            let mut outcomes = Vec::with_capacity(rows.len());
            for row in rows {
                let object = row.as_object().expect("row shape checked above");
                let key_id = object
                    .get(&self.key_id_field)
                    .and_then(Value::as_str)
                    .ok_or_else(|| PlusError::Interceptor {
                        stage: InterceptorStage::ResultVerify,
                        message: format!("missing signature key field `{}`", self.key_id_field),
                    })?;
                let digest = object
                    .get(&self.signature_field)
                    .and_then(Value::as_str)
                    .ok_or_else(|| PlusError::Interceptor {
                        stage: InterceptorStage::ResultVerify,
                        message: format!("missing signature field `{}`", self.signature_field),
                    })?;
                let mut payload = object.clone();
                payload.remove(&self.key_id_field);
                payload.remove(&self.signature_field);
                let outcome = self.service.verify(
                    &Value::Object(payload),
                    &expected,
                    &signed,
                    self.scope,
                    &RowSignature {
                        key_id: key_id.to_owned(),
                        digest: digest.to_owned(),
                    },
                    self.partial_policy,
                )?;
                outcomes.push(format!("{outcome:?}"));
            }
            invocation.attributes.insert(
                "row_signature.outcomes".to_owned(),
                Value::Array(outcomes.into_iter().map(Value::String).collect()),
            );
            Ok(())
        })
    }
}

impl RowSignatureService {
    pub fn new(
        active_key_id: impl Into<String>,
        keys: impl IntoIterator<Item = (String, Vec<u8>)>,
    ) -> PlusResult<Self> {
        let active_key_id = active_key_id.into();
        let keys = keys
            .into_iter()
            .map(|(id, key)| (id, Zeroizing::new(key)))
            .collect::<BTreeMap<_, _>>();
        if keys.get(&active_key_id).is_none_or(|key| key.is_empty()) {
            return Err(PlusError::InvalidArgument(
                "active signature key is missing or empty".to_owned(),
            ));
        }
        if keys.values().any(|key| key.is_empty()) {
            return Err(PlusError::InvalidArgument(
                "signature keys must not be empty".to_owned(),
            ));
        }
        Ok(Self {
            active_key_id,
            keys,
        })
    }

    pub fn sign(
        &self,
        row: &Value,
        signed_columns: &[&str],
        scope: SignatureScope,
    ) -> PlusResult<RowSignature> {
        let payload = signature_payload(row, signed_columns, scope)?;
        let digest = self.sign_with(&self.active_key_id, &payload)?;
        Ok(RowSignature {
            key_id: self.active_key_id.clone(),
            digest,
        })
    }

    pub fn verify(
        &self,
        row: &Value,
        expected_columns: &[&str],
        signed_columns: &[&str],
        scope: SignatureScope,
        signature: &RowSignature,
        partial_policy: PartialRowPolicy,
    ) -> PlusResult<VerificationOutcome> {
        let object = row.as_object().ok_or_else(|| {
            PlusError::InvalidArgument("signed row must be a JSON object".to_owned())
        })?;
        let missing = expected_columns
            .iter()
            .copied()
            .filter(|column| !object.contains_key(*column))
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return match partial_policy {
                PartialRowPolicy::RejectPartial => Err(PlusError::Interceptor {
                    stage: InterceptorStage::ResultVerify,
                    message: format!("partial signed row is missing: {}", missing.join(", ")),
                }),
                PartialRowPolicy::DeferredResign => Ok(VerificationOutcome::DeferredResign),
            };
        }
        let payload = signature_payload(row, signed_columns, scope)?;
        let key = self
            .keys
            .get(&signature.key_id)
            .ok_or_else(|| PlusError::Interceptor {
                stage: InterceptorStage::ResultVerify,
                message: format!("unknown row signature key `{}`", signature.key_id),
            })?;
        let supplied = URL_SAFE_NO_PAD
            .decode(&signature.digest)
            .map_err(crypto_error)?;
        let mut mac = <HmacSha256 as Mac>::new_from_slice(key.as_ref()).map_err(crypto_error)?;
        mac.update(&payload);
        mac.verify_slice(&supplied)
            .map_err(|_| PlusError::Interceptor {
                stage: InterceptorStage::ResultVerify,
                message: "row signature verification failed".to_owned(),
            })?;
        if signature.key_id == self.active_key_id {
            Ok(VerificationOutcome::Verified)
        } else {
            Ok(VerificationOutcome::VerifiedNeedsResign)
        }
    }

    fn sign_with(&self, key_id: &str, payload: &[u8]) -> PlusResult<String> {
        let key = self.keys.get(key_id).ok_or_else(|| {
            PlusError::InvalidArgument(format!("unknown signature key `{key_id}`"))
        })?;
        let mut mac = <HmacSha256 as Mac>::new_from_slice(key.as_ref()).map_err(crypto_error)?;
        mac.update(payload);
        Ok(URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes()))
    }
}

fn decrypt_result<C: FieldCipher>(
    cipher: &C,
    fields: &BTreeMap<String, Vec<u8>>,
    value: &mut Value,
) -> PlusResult<()> {
    match value {
        Value::Array(rows) => {
            for row in rows {
                decrypt_result(cipher, fields, row)?;
            }
        }
        Value::Object(row) => {
            for (field, context) in fields {
                if let Some(Value::String(envelope)) = row.get_mut(field) {
                    let plaintext = cipher.decrypt(envelope, context)?;
                    *envelope = String::from_utf8(plaintext).map_err(|error| {
                        PlusError::InvalidArgument(format!(
                            "decrypted field `{field}` is not UTF-8: {error}"
                        ))
                    })?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn signature_payload(
    row: &Value,
    signed_columns: &[&str],
    scope: SignatureScope,
) -> PlusResult<Vec<u8>> {
    let object = row
        .as_object()
        .ok_or_else(|| PlusError::InvalidArgument("signed row must be a JSON object".to_owned()))?;
    let value = match scope {
        SignatureScope::FullRow => canonicalize(row),
        SignatureScope::SignatureOnly => {
            let columns = signed_columns.iter().copied().collect::<BTreeSet<_>>();
            let mut selected = Map::new();
            for column in columns {
                let value = object.get(column).ok_or_else(|| PlusError::Interceptor {
                    stage: InterceptorStage::ResultVerify,
                    message: format!("signed column `{column}` is missing"),
                })?;
                selected.insert(column.to_owned(), canonicalize(value));
            }
            Value::Object(selected)
        }
    };
    serde_json::to_vec(&value).map_err(crypto_error)
}

fn canonicalize(value: &Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.iter().map(canonicalize).collect()),
        Value::Object(values) => {
            let sorted = values.iter().collect::<BTreeMap<_, _>>();
            Value::Object(
                sorted
                    .into_iter()
                    .map(|(key, value)| (key.clone(), canonicalize(value)))
                    .collect(),
            )
        }
        value => value.clone(),
    }
}

fn crypto_error(error: impl std::fmt::Display) -> PlusError {
    PlusError::InvalidArgument(format!("cryptographic operation failed: {error}"))
}
