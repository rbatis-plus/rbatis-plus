//! Condition builders for RBatis-Plus.
//!
//! Mirrors `mybatis-plus-core/.../conditions/`.

pub mod abstract_wrapper;
pub mod compare;
pub mod func;
pub mod is_sql_segment;
pub mod join;
pub mod merge_segments;
pub mod nested;
pub mod query;
pub mod shared_string;
pub mod update;

pub use abstract_wrapper::AbstractWrapper;
pub use compare::Compare;
pub use func::{Func, FuncSegments};
pub use is_sql_segment::{ISqlSegment, SqlType};
pub use join::Join;
pub use merge_segments::MergeSegments;
pub use nested::Nested;
pub use shared_string::SharedString;
pub use query::{Column, LambdaColumns, LambdaQueryWrapper, QueryWrapper};
pub use update::{LambdaUpdateWrapper, UpdateWrapper};
