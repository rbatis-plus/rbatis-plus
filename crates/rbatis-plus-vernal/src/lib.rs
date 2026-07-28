//! RBatis-Plus Web 框架集成层（对标 mybatis-plus-spring + mybatis-plus-enhance-spring）。
//!
//! 提供 axum/actix-web 框架的集成能力：
//! - RBatis 应用状态管理
//! - 事务中间件
//! - 分页请求提取器
//! - 自动配置辅助
//! - 原生 SQL 执行器（SqlRunner）
//! - RAII 事务守卫
//!
//! # 对应 Java
//!
//! - `com.baomidou.mybatisplus.autoconfigure.MybatisPlusAutoConfiguration`
//! - `com.baomidou.mybatisplus.extension.spring.MybatisSqlSessionFactoryBean`
//! - `com.baomidou.mybatisplus.extension.plugins.inner.PaginationInnerInterceptor`
//! - `com.baomidou.mybatisplus.extension.SqlRunner`
//! - `com.baomidou.mybatisplus.extension.toolkit.TransactionTemplate`

pub mod config;
pub mod sql_runner;
pub mod state;
pub mod transaction;

#[cfg(feature = "actix")]
pub mod actix_integration;

#[cfg(feature = "axum")]
pub mod axum_integration;

pub use config::VernalConfig;
pub use sql_runner::SqlRunner;
pub use state::AppState;
pub use transaction::{run_in_transaction, TransactionalGuard};
