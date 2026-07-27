//! Inner interceptors for RBatis-Plus.
//!
//! Mirrors `mybatis-plus/.../inner/` and `mybatis-plus-enhance/.../plugins/inner/`.

pub mod base;
pub mod block_attack;
pub mod data_permission;
pub mod dynamic_table_name;
pub mod enhance_phase;
pub mod enhance_interceptor;
pub mod inner_interceptor;
pub mod mybatis_plus_enhance_interceptor;
pub mod optimistic_locker;
pub mod pagination;
pub mod tenant;

// 短名别名（显式命名导出，禁止 wildcard）
pub use base::BlockAttack;
pub use base::DataPermission;
pub use base::DynamicTableName;
pub use base::OptimisticLocker;
pub use base::Pagination;
pub use base::Tenant;
pub use block_attack::BlockAttackInnerInterceptor;
pub use data_permission::{DataPermissionHandler, DataPermissionInnerInterceptor};
pub use dynamic_table_name::{
    DynamicTableNameInnerInterceptor, TableNameHandler as DynamicTableNameHandler,
};
pub use enhance_phase::EnhancePhase;
pub use enhance_interceptor::EnhanceInnerInterceptor;
pub use inner_interceptor::InnerInterceptor;
pub use mybatis_plus_enhance_interceptor::MybatisPlusEnhanceInterceptor;
pub use optimistic_locker::OptimisticLockerInnerInterceptor;
pub use pagination::PaginationInnerInterceptor;
pub use tenant::{TenantInnerInterceptor, TenantLineHandler};
