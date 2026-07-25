//! Condition builders for RBatis-Plus.
//!
//! Mirrors `mybatis-plus-core/.../conditions/`.

pub mod abstract_wrapper;
pub mod compare;
pub mod func;
pub mod merge_segments;
pub mod nested;
pub mod query;
pub mod update;

pub use abstract_wrapper::AbstractWrapper;
pub use compare::Compare;
pub use func::{Func, FuncSegments};
pub use merge_segments::MergeSegments;
pub use nested::{Nested, Join};
pub use query::{Column, LambdaColumns, LambdaQueryWrapper, QueryWrapper};
pub use update::{LambdaUpdateWrapper, UpdateWrapper};
