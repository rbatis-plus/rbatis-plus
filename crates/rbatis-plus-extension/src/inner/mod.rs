//! Inner interceptors for RBatis-Plus.
//!
//! Mirrors `mybatis-plus/.../inner/` and `mybatis-plus-enhance/.../plugins/inner/`.

pub mod base;
pub mod block_attack;
pub mod data_permission;
pub mod dynamic_table_name;
pub mod inner_interceptor;
pub mod optimistic_locker;
pub mod pagination;
pub mod tenant;

pub use base::*;
pub use block_attack::BlockAttackInnerInterceptor;
pub use data_permission::{DataPermissionHandler, DataPermissionInnerInterceptor};
pub use dynamic_table_name::{
    DynamicTableNameInnerInterceptor, TableNameHandler as DynamicTableNameHandler,
};
pub use inner_interceptor::InnerInterceptor;
pub use optimistic_locker::OptimisticLockerInnerInterceptor;
pub use pagination::PaginationInnerInterceptor;
pub use tenant::{TenantInnerInterceptor, TenantLineHandler};
