//! Core conditions, metadata, mapper traits, toolkit and page for RBatis-Plus.

pub mod conditions;
pub mod derive;
pub mod mapper;
pub mod metadata;
pub mod page;
pub mod toolkit;
pub mod wrapper;

/// 二级缓存子系统：CacheStore SPI、CacheKey、CachePolicy、CacheIntercept、
/// CacheTransactionListener 以及内置 MemoryCacheStore。
pub mod cache {
    pub mod error;
    pub mod intercept;
    pub mod key;
    pub mod listener;
    pub mod memory;
    pub mod policy;
    pub mod store;

    pub use error::CacheError;
    pub use intercept::CacheIntercept;
    pub use key::CacheKey;
    pub use listener::CacheTransactionListener;
    pub use memory::MemoryCacheStore;
    pub use policy::*;
    pub use store::{CacheStore, CacheTag};
}

pub use conditions::*;
pub use derive::*;
pub use mapper::BaseMapper;
pub use metadata::{TableFieldInfo, TableInfo};
pub use page::{Page, PageRequest};
