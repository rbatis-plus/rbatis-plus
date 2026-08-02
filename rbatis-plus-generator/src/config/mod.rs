//! 配置模块（对标 mybatis-plus-generator `config` 包）。

pub mod data_source;
pub mod global;
pub mod package;
pub mod strategy;

pub use data_source::DataSourceConfig;
pub use global::GlobalConfig;
pub use package::PackageConfig;
pub use strategy::StrategyConfig;
