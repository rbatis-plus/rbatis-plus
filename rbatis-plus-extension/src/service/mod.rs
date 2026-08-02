//! Service layer traits for RBatis-Plus.
//!
//! Mirrors `mybatis-plus-extension/.../repository/` and
//! `mybatis-plus-spring/.../service/`.

pub mod i_service;
pub mod service_impl;

pub use i_service::IService;
pub use service_impl::ServiceImpl;
