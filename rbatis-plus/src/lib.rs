#![forbid(unsafe_code)]
//! Facade exports for rbatis-plus applications.

pub use rbatis_plus_core::*;
pub use rbatis_plus_extension::*;
pub use rbatis_plus_macros::PlusModel;

pub mod prelude {
    pub use rbatis_plus_core::{
        BaseMapper, Column, IService, Page, PageRequest, QueryWrapper, ServiceImpl, SortDirection,
        TableMetadata, UpdateWrapper,
    };
    pub use rbatis_plus_macros::PlusModel;
}
