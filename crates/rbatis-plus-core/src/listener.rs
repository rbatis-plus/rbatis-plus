use crate::CacheStore;
use async_trait::async_trait;
use rbatis::plugin::transaction::{TransactionEvent, TransactionEventType, TransactionListener};
use rbatis::Error;
use std::sync::Arc;

/// Transaction listener that invalidates the cache namespace after a
/// successful commit.
///
/// This implements the "commit-aware invalidation" strategy described in
/// `TRANSACTION_CONSISTENCY.md`.  On `CommitSuccess`, it clears the
/// namespace so that any data written during the transaction is visible
/// to subsequent cache reads.
///
/// On `Rollback`, it does **nothing** — the cache was bypassed during the
/// transaction (by `CacheIntercept`), so there's nothing to roll back.
#[derive(Clone)]
pub struct CacheTransactionListener {
    pub store: Arc<dyn CacheStore>,
    pub namespace: String,
}

impl CacheTransactionListener {
    pub fn new(store: Arc<dyn CacheStore>, namespace: impl Into<String>) -> Self {
        Self {
            store,
            namespace: namespace.into(),
        }
    }
}

impl std::fmt::Debug for CacheTransactionListener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheTransactionListener")
            .field("namespace", &self.namespace)
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl TransactionListener for CacheTransactionListener {
    async fn on_event(&self, event: &TransactionEvent) -> Result<(), Error> {
        match event.event_type {
            TransactionEventType::CommitSuccess => {
                log::debug!(
                    "rbatis-plus: commit-success, invalidating namespace '{}' (tx={})",
                    self.namespace,
                    event.tx_id
                );
                if let Err(e) = self.store.clear_namespace(&self.namespace).await {
                    log::warn!(
                        "rbatis-plus: post-commit invalidation failed (fail-open): {}",
                        e
                    );
                }
            }
            TransactionEventType::CommitFailed => {
                log::debug!(
                    "rbatis-plus: commit-failed (tx={}), cache untouched",
                    event.tx_id
                );
            }
            TransactionEventType::Rollback => {
                log::debug!(
                    "rbatis-plus: rollback (tx={}), cache untouched (was bypassed)",
                    event.tx_id
                );
            }
            _ => {}
        }
        Ok(())
    }
}
