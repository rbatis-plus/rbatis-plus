//! Inner interceptors for RBatis-Plus.
//!
//! Mirrors `mybatis-plus/.../inner/` and `mybatis-plus-enhance/.../plugins/inner/`.

pub mod base;
pub mod block_attack;
pub mod data_decryption;
pub mod data_encryption;
pub mod data_i18n;
pub mod data_permission;
pub mod data_signature;
pub mod dynamic_table_name;
pub mod enhance_phase;
pub mod enhance_interceptor;
pub mod inner_interceptor;
pub mod long_sql;
pub mod mybatis_plus_enhance_interceptor;
pub mod optimistic_locker;
pub mod pagination;
pub mod sql_observation;
pub mod tenant;

// 短名别名（显式命名导出，禁止 wildcard）
pub use base::BlockAttack;
pub use base::DataPermission;
pub use base::DynamicTableName;
pub use base::OptimisticLocker;
pub use base::Pagination;
pub use base::Tenant;
pub use block_attack::BlockAttackInnerInterceptor;
pub use data_decryption::DataDecryptionInnerInterceptor;
pub use data_encryption::DataEncryptionInnerInterceptor;
pub use data_i18n::DataI18nInnerInterceptor;
pub use data_permission::{DataPermissionHandler, DataPermissionInnerInterceptor};
pub use data_signature::DataSignatureInnerInterceptor;
pub use dynamic_table_name::{
    DynamicTableNameInnerInterceptor, TableNameHandler as DynamicTableNameHandler,
};
pub use enhance_phase::EnhancePhase;
pub use enhance_interceptor::EnhanceInnerInterceptor;
pub use inner_interceptor::InnerInterceptor;
pub use long_sql::{LongSqlHandler, LongSqlInnerInterceptor};
pub use mybatis_plus_enhance_interceptor::MybatisPlusEnhanceInterceptor;
pub use optimistic_locker::OptimisticLockerInnerInterceptor;
pub use pagination::PaginationInnerInterceptor;
pub use sql_observation::{SqlObservation, SqlObservationInnerInterceptor, SqlObservationSink};
pub use tenant::{TenantInnerInterceptor, TenantLineHandler};
