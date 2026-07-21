use rbatis_plus_core::{InterceptorChain, InterceptorStage, SqlInvocation};
use rbatis_plus_extension::{
    AesGcmKeyRing, EncryptedParameter, FieldCipher, FieldDecryptionInterceptor,
    FieldEncryptionInterceptor, PartialRowPolicy, RowSignatureService,
    RowSignatureVerificationInterceptor, RowVerificationConfig, SecurePipelineBuilder,
    SignatureScope, VerificationOutcome,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::Arc;

fn cipher(active: &str) -> AesGcmKeyRing {
    AesGcmKeyRing::new(
        active,
        [("old".to_owned(), [7; 32]), ("current".to_owned(), [9; 32])],
        [11; 32],
    )
    .unwrap()
}

#[test]
fn encrypts_with_random_nonces_authenticates_context_and_builds_stable_blind_indexes() {
    let cipher = cipher("current");
    let first = cipher.encrypt(b"13800138000", b"orders.phone").unwrap();
    let second = cipher.encrypt(b"13800138000", b"orders.phone").unwrap();
    assert_ne!(first, second);
    assert_eq!(
        cipher.decrypt(&first, b"orders.phone").unwrap(),
        b"13800138000"
    );
    assert!(cipher.decrypt(&first, b"users.phone").is_err());

    let blind = cipher.blind_index(b"13800138000", b"orders.phone").unwrap();
    assert_eq!(blind, "fkTSqUy7p-B7y2JLvyLosQ6A2q22e85gJ6YpY89Ar9g");
    assert_ne!(
        blind,
        cipher.blind_index(b"13800138001", b"orders.phone").unwrap()
    );
}

#[tokio::test]
async fn encryption_and_decryption_run_in_fixed_pipeline_stages() {
    let cipher = Arc::new(cipher("current"));
    let chain = InterceptorChain::new(vec![
        Arc::new(FieldDecryptionInterceptor::new(
            cipher.clone(),
            BTreeMap::from([("phone".to_owned(), b"orders.phone".to_vec())]),
        )),
        Arc::new(FieldEncryptionInterceptor::new(
            cipher,
            vec![EncryptedParameter {
                index: 0,
                context: b"orders.phone".to_vec(),
            }],
        )),
    ]);
    assert_eq!(
        chain.stages(),
        [
            InterceptorStage::ParameterTransform,
            InterceptorStage::ResultTransform
        ]
    );
    let mut invocation = SqlInvocation::new(
        "OrderMapper.insert",
        "INSERT INTO orders(phone) VALUES (?)",
        vec![Value::String("13800138000".to_owned())],
    );
    chain.apply(&mut invocation).await.unwrap();
    let encrypted = invocation.parameters[0].as_str().unwrap().to_owned();
    assert!(encrypted.starts_with("v1.current."));

    invocation.result = Some(json!([{"phone": encrypted}]));
    chain.apply(&mut invocation).await.unwrap();
    assert_eq!(invocation.result.unwrap()[0]["phone"], "13800138000");
}

#[tokio::test]
async fn verifies_ciphertext_signature_before_decrypting_result_fields() {
    let cipher = Arc::new(cipher("current"));
    let encrypted = cipher.encrypt(b"13800138000", b"orders.phone").unwrap();
    let signer = Arc::new(
        RowSignatureService::new("current", [("current".to_owned(), vec![5; 32])]).unwrap(),
    );
    let stored_row = json!({"id": 1, "phone": encrypted});
    let signature = signer
        .sign(&stored_row, &["id", "phone"], SignatureScope::FullRow)
        .unwrap();
    let chain = InterceptorChain::new(vec![
        Arc::new(FieldDecryptionInterceptor::new(
            cipher,
            BTreeMap::from([("phone".to_owned(), b"orders.phone".to_vec())]),
        )),
        Arc::new(RowSignatureVerificationInterceptor::new(
            signer,
            vec!["id".to_owned(), "phone".to_owned()],
            vec!["id".to_owned(), "phone".to_owned()],
            SignatureScope::FullRow,
            PartialRowPolicy::RejectPartial,
            "signature_key",
            "signature",
        )),
    ]);
    assert_eq!(
        chain.stages(),
        [
            InterceptorStage::ResultVerify,
            InterceptorStage::ResultTransform
        ]
    );
    let mut invocation = SqlInvocation::new("OrderMapper.select", "SELECT", vec![]);
    invocation.result = Some(json!({
        "id": 1,
        "phone": stored_row["phone"],
        "signature_key": signature.key_id,
        "signature": signature.digest
    }));
    chain.apply(&mut invocation).await.unwrap();
    assert_eq!(invocation.result.unwrap()["phone"], "13800138000");
}

#[tokio::test]
async fn secure_builder_fixes_stage_order_and_scopes_parameter_encryption() {
    let cipher = Arc::new(cipher("current"));
    let signer = Arc::new(
        RowSignatureService::new("current", [("current".to_owned(), vec![5; 32])]).unwrap(),
    );
    let verification = RowVerificationConfig::new(
        signer,
        vec!["id".to_owned(), "phone".to_owned()],
        vec!["id".to_owned(), "phone".to_owned()],
        SignatureScope::FullRow,
        PartialRowPolicy::RejectPartial,
        "signature_key",
        "signature",
    )
    .unwrap();
    let chain = SecurePipelineBuilder::new(cipher, verification)
        .encrypt_parameters_for(
            ["OrderMapper.insert"],
            vec![EncryptedParameter {
                index: 0,
                context: b"orders.phone".to_vec(),
            }],
        )
        .decrypt_fields(BTreeMap::from([(
            "phone".to_owned(),
            b"orders.phone".to_vec(),
        )]))
        .build()
        .unwrap();
    assert_eq!(
        chain.stages(),
        [
            InterceptorStage::ParameterTransform,
            InterceptorStage::ResultVerify,
            InterceptorStage::ResultTransform,
        ]
    );

    let mut select = SqlInvocation::new("OrderMapper.select", "SELECT * FROM orders", vec![]);
    chain.apply_before_execute(&mut select).await.unwrap();

    let mut insert = SqlInvocation::new(
        "OrderMapper.insert",
        "INSERT INTO orders(phone) VALUES (?)",
        vec![Value::String("13800138000".to_owned())],
    );
    chain.apply_before_execute(&mut insert).await.unwrap();
    assert!(
        insert.parameters[0]
            .as_str()
            .unwrap()
            .starts_with("v1.current.")
    );
}

#[test]
fn secure_builder_rejects_incomplete_fail_open_configuration() {
    let signer = Arc::new(
        RowSignatureService::new("current", [("current".to_owned(), vec![5; 32])]).unwrap(),
    );
    assert!(
        RowVerificationConfig::new(
            signer.clone(),
            Vec::new(),
            Vec::new(),
            SignatureScope::FullRow,
            PartialRowPolicy::RejectPartial,
            "signature_key",
            "signature",
        )
        .is_err()
    );
    let verification = RowVerificationConfig::new(
        signer,
        vec!["id".to_owned()],
        vec!["id".to_owned()],
        SignatureScope::FullRow,
        PartialRowPolicy::RejectPartial,
        "signature_key",
        "signature",
    )
    .unwrap();
    assert!(
        SecurePipelineBuilder::new(Arc::new(cipher("current")), verification)
            .build()
            .is_err()
    );
}

#[test]
fn row_signatures_fail_closed_support_rotation_and_enforce_partial_policies() {
    let old = RowSignatureService::new("old", [("old".to_owned(), vec![3; 32])]).unwrap();
    let row = json!({"id": 1, "name": "alpha", "tenant": "tenant-a"});
    let signature = old
        .sign(&row, &["id", "name", "tenant"], SignatureScope::FullRow)
        .unwrap();
    let rotated = RowSignatureService::new(
        "current",
        [
            ("old".to_owned(), vec![3; 32]),
            ("current".to_owned(), vec![5; 32]),
        ],
    )
    .unwrap();
    assert_eq!(
        rotated
            .verify(
                &row,
                &["id", "name", "tenant"],
                &["id", "name", "tenant"],
                SignatureScope::FullRow,
                &signature,
                PartialRowPolicy::RejectPartial,
            )
            .unwrap(),
        VerificationOutcome::VerifiedNeedsResign
    );

    let tampered = json!({"id": 1, "name": "tampered", "tenant": "tenant-a"});
    assert!(
        rotated
            .verify(
                &tampered,
                &["id", "name", "tenant"],
                &["id", "name", "tenant"],
                SignatureScope::FullRow,
                &signature,
                PartialRowPolicy::RejectPartial,
            )
            .is_err()
    );

    let partial = json!({"id": 1, "name": "alpha"});
    assert!(
        rotated
            .verify(
                &partial,
                &["id", "name", "tenant"],
                &["id", "name", "tenant"],
                SignatureScope::SignatureOnly,
                &signature,
                PartialRowPolicy::RejectPartial,
            )
            .is_err()
    );
    assert_eq!(
        rotated
            .verify(
                &partial,
                &["id", "name", "tenant"],
                &["id", "name", "tenant"],
                SignatureScope::SignatureOnly,
                &signature,
                PartialRowPolicy::DeferredResign,
            )
            .unwrap(),
        VerificationOutcome::DeferredResign
    );
}
