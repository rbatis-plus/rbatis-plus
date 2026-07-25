// Re-export aliases for convenience.
//
// Some users prefer shorter names; this module provides them.

pub use super::block_attack::BlockAttackInnerInterceptor as BlockAttack;
pub use super::data_permission::DataPermissionInnerInterceptor as DataPermission;
pub use super::dynamic_table_name::DynamicTableNameInnerInterceptor as DynamicTableName;
pub use super::optimistic_locker::OptimisticLockerInnerInterceptor as OptimisticLocker;
pub use super::pagination::PaginationInnerInterceptor as Pagination;
pub use super::tenant::TenantInnerInterceptor as Tenant;
