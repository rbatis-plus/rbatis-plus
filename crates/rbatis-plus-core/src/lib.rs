//! Core conditions, metadata, mapper traits, toolkit and page for RBatis-Plus.

pub mod conditions;
pub mod derive;
pub mod mapper;
pub mod metadata;
pub mod method;
pub mod page;
pub mod toolkit;
pub mod wrapper;

/// 二级缓存子系统：复用上游 rbatis::plugin::cache 的完整实现。
pub mod cache;

// Re-exports from conditions（禁止 wildcard）
pub use conditions::abstract_wrapper::AbstractWrapper;
pub use conditions::compare::Compare;
pub use conditions::func::{Func, FuncSegments};
pub use conditions::merge_segments::MergeSegments;
pub use conditions::nested::Nested;
pub use conditions::join::Join;
pub use conditions::shared_string::SharedString;
pub use conditions::is_sql_segment::{ISqlSegment, SqlType};
pub use conditions::query::{Column, LambdaColumns, LambdaQueryWrapper, QueryWrapper};
pub use conditions::update::{LambdaUpdateWrapper, UpdateWrapper};
// Re-exports from derive（禁止 wildcard）
pub use derive::{
    EncryptedFieldAttr, EncryptedTable, FieldFill, FieldStrategy,
    I18nColumnAttr, I18nColumn, IdType, SignatureFieldAttr,
    DbType, TableFieldAttr, TableId, TableLogic, TableName, TableNameInfo, TableSignature, Version,
    IEnum, OrderBy, KeySequence, InterceptorIgnore, InterceptorIgnoreInfo,
};
pub use mapper::BaseMapper;
pub use metadata::{TableFieldInfo, TableInfo};
pub use page::{Page, PageRequest};
