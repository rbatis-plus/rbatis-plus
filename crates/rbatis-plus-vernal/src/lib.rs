//! RBatis-Plus Web 框架集成层（对标 mybatis-plus-spring + mybatis-plus-enhance-spring）。
//!
//! 提供 axum/actix-web 框架的集成能力：
//! - RBatis 应用状态管理
//! - 事务中间件
//! - 分页请求提取器
//! - 自动配置辅助
//!
//! # 对应 Java
//!
//! - `com.baomidou.mybatisplus.autoconfigure.MybatisPlusAutoConfiguration`
//! - `com.baomidou.mybatisplus.extension.spring.MybatisSqlSessionFactoryBean`
//! - `com.baomidou.mybatisplus.extension.plugins.inner.PaginationInnerInterceptor`

pub mod axum_integration;
pub mod config;
pub mod state;

pub use config::VernalConfig;
pub use state::AppState;
