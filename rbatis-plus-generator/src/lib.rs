//! RBatis-Plus 代码生成器（对标 mybatis-plus-generator）。
//!
//! 自动生成 Entity、Mapper、Service、Controller 代码，
//! 基于数据库表元数据查询 + Tera 模板引擎。
//!
//! **注意**：lib.rs 禁止定义对象；所有类型在独立文件中定义，这里只做声明与 re-export。
//!
//! # 快速开始
//!
//! ```ignore
//! use rbatis_plus_generator::*;
//!
//! let generator = AutoGenerator::builder()
//!     .data_source(
//!         DataSourceConfig::builder()
//!             .url("mysql://root:123456@localhost:3306/test")
//!             .username("root")
//!             .password("123456")
//!             .build()
//!     )
//!     .global(
//!         GlobalConfig::builder()
//!             .output_dir("./generated")
//!             .author("rbatis-plus")
//!             .build()
//!     )
//!     .package(
//!         PackageConfig::builder()
//!             .parent("crate")
//!             .module_name("user")
//!             .build()
//!     )
//!     .strategy(
//!         StrategyConfig::builder()
//!             .include(vec!["sys_user", "sys_role"])
//!             .table_prefix(vec!["sys_"])
//!             .build()
//!     )
//!     .build();
//!
//! generator.execute().await?;
//! ```

pub mod auto_generator;
pub mod auto_generator_builder;
pub mod config;
pub mod engine;
pub mod query;
pub mod template;

// Re-export 核心类型（禁止 wildcard）
pub use auto_generator::AutoGenerator;
pub use auto_generator_builder::AutoGeneratorBuilder;
pub use config::{DataSourceConfig, GlobalConfig, PackageConfig, StrategyConfig};
pub use config::data_source::DbType;
pub use engine::TeraEngine;
pub use query::{TableField, TableInfo};
pub use template::{AskamaEngine, HandlebarsEngine, MaudEngine, TemplateEngine};
