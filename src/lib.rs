//! RBatis-Plus — MyBatis-Plus equivalent enhancement framework for RBatis.
//!
//! # Quick start
//!
//! ```ignore
//! use rbatis_plus::QueryWrapper;
//! use rbatis::RBatis;
//!
//! let rb = RBatis::new();
//! // rb.init(driver, url)?;
//!
//! let users: Vec<User> = QueryWrapper::new()
//!     .eq("name", "Alice")
//!     .ge("age", 18)
//!     .query(&rb, "sys_user")
//!     .await?;
//! ```

pub use rbatis_plus_core as core;
pub use rbatis_plus_extension as extension;
pub use rbatis_plus_generator as generator;
pub use rbatis_plus_sqlparser as sqlparser;
pub use rbatis_plus_vernal as vernal;

// Re-export the most commonly used types
pub use rbatis_plus_core::conditions::query::QueryWrapper;
pub use rbatis_plus_core::conditions::query::LambdaQueryWrapper;
pub use rbatis_plus_core::conditions::query::Column;
pub use rbatis_plus_core::conditions::update::UpdateWrapper;
pub use rbatis_plus_core::conditions::update::LambdaUpdateWrapper;
pub use rbatis_plus_core::conditions::{Compare, Func, Join, Nested};
pub use rbatis_plus_core::mapper::BaseMapper;
pub use rbatis_plus_core::metadata::{TableFieldInfo, TableInfo};
pub use rbatis_plus_core::page::{Page, PageRequest};

pub use rbatis_plus_extension::inner::data_permission::DataPermissionInnerInterceptor;
pub use rbatis_plus_extension::inner::data_i18n::DataI18nInnerInterceptor;
pub use rbatis_plus_extension::inner::InnerInterceptor;
pub use rbatis_plus_extension::inner::block_attack::BlockAttackInnerInterceptor;
pub use rbatis_plus_extension::inner::pagination::PaginationInnerInterceptor;
pub use rbatis_plus_extension::inner::tenant::{TenantInnerInterceptor, TenantLineHandler};
pub use rbatis_plus_extension::inner::optimistic_locker::OptimisticLockerInnerInterceptor;
pub use rbatis_plus_extension::inner::dynamic_table_name::DynamicTableNameInnerInterceptor;
pub use rbatis_plus_extension::service::IService;
pub use rbatis_plus_extension::service::ServiceImpl;

// 缓存类型 re-export（实现位于 rbatis-cache crate）。
#[cfg(feature = "cache")]
pub use rbatis_cache::{CacheBackend, CacheError, CacheEnvelope, CacheFailureMode, CacheInterceptor, CacheKey, CacheKeyInput, CacheMetrics, CacheMetricsSnapshot, CachePolicy, CacheRequest, CacheTransactionListener, InvalidationStrategy, L1Cache, LocalBackend, LocalBackendConfig, RbatisCacheExt, RbatisCacheInterceptor, SingleFlight, SqlMetadata, StatementKind, TransactionCacheMode, TransactionalCacheBuffer, UseCacheFilter};
