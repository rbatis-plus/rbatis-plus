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
    // Re-export from cache::policy（禁止 wildcard）
    pub use policy::{CachePolicy, TransactionMode, FailureMode};
    pub use store::{CacheStore, CacheTag};
}

// Re-exports from conditions（禁止 wildcard）
pub use conditions::abstract_wrapper::AbstractWrapper;
pub use conditions::compare::Compare;
pub use conditions::func::{Func, FuncSegments};
pub use conditions::merge_segments::MergeSegments;
pub use conditions::nested::{Nested, Join};
pub use conditions::query::{Column, LambdaColumns, LambdaQueryWrapper, QueryWrapper};
pub use conditions::update::{LambdaUpdateWrapper, UpdateWrapper};
// Re-exports from derive（禁止 wildcard）
pub use derive::{
    EncryptedFieldAttr, EncryptedTable, FieldFill, FieldStrategy,
    I18nColumnAttr, I18nColumn, IdType, SignatureFieldAttr,
    TableFieldAttr, TableId, TableLogic, TableName, TableNameInfo, TableSignature, Version,
    OrderBy, KeySequence, InterceptorIgnore, InterceptorIgnoreInfo,
};
pub use mapper::BaseMapper;
pub use metadata::{TableFieldInfo, TableInfo};
pub use page::{Page, PageRequest};
