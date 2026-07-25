//! RBatis-Plus 代码生成器（对标 mybatis-plus-generator）。
//!
//! 自动生成 Entity、Mapper、Service、Controller 代码，
//! 基于数据库表元数据查询 + Tera 模板引擎。
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

pub mod config;
pub mod engine;
pub mod query;

// Re-export 核心类型
pub use config::{DataSourceConfig, GlobalConfig, PackageConfig, StrategyConfig};
pub use config::data_source::DbType;
pub use engine::TeraEngine;
pub use query::{TableField, TableInfo};

/// 代码生成器主入口（对标 Java `com.baomidou.mybatisplus.generator.AutoGenerator`）。
///
/// 负责协调配置加载、元数据查询、模板渲染、文件输出的完整流程。
pub struct AutoGenerator {
    pub data_source: DataSourceConfig,
    pub global: GlobalConfig,
    pub package: PackageConfig,
    pub strategy: StrategyConfig,
}

impl AutoGenerator {
    /// 创建构建器。
    pub fn builder() -> AutoGeneratorBuilder {
        AutoGeneratorBuilder::default()
    }

    /// 执行代码生成（对标 `AutoGenerator.execute()`）。
    ///
    /// 1. 连接数据库查询表元数据
    /// 2. 过滤表（include/exclude）
    /// 3. 使用 Tera 引擎渲染代码
    /// 4. 输出文件到指定目录
    pub async fn execute(&self) -> Result<Vec<std::path::PathBuf>, String> {
        log::info!("RBatis-Plus Generator 开始执行...");
        log::info!("数据源: {}", self.data_source.url);
        log::info!("输出目录: {}", self.global.output_dir);

        // 查询表元数据
        let tables = self.query_tables().await?;
        log::info!("查询到 {} 张表", tables.len());

        // 过滤表
        let filtered: Vec<_> = tables
            .into_iter()
            .filter(|t| self.strategy.is_table_included(&t.name))
            .collect();
        log::info!("过滤后 {} 张表需要生成", filtered.len());

        if filtered.is_empty() {
            log::warn!("没有需要生成的表，请检查 include/exclude 配置");
            return Ok(Vec::new());
        }

        // 使用 Tera 引擎生成代码
        let engine = TeraEngine::new(
            self.global.clone(),
            self.package.clone(),
            self.strategy.clone(),
        );

        let files = engine.batch_output(&filtered)?;
        log::info!("代码生成完成，共生成 {} 个文件", files.len());

        Ok(files)
    }

    /// 查询数据库表元数据（对标 `DefaultQuery.queryTables()`）。
    ///
    /// 目前返回模拟数据；生产版本应通过 rbdc 连接数据库查询 `information_schema`。
    async fn query_tables(&self) -> Result<Vec<TableInfo>, String> {
        // TODO: 通过 rbdc 连接数据库查询 information_schema
        // 当前返回空列表，用户可通过 `execute_with_tables()` 直接传入表元数据
        log::warn!("当前版本暂不支持自动查询数据库元数据，请使用 `execute_with_tables()` 手动传入");
        Ok(Vec::new())
    }

    /// 使用手动提供的表元数据执行生成（用于不支持自动查询的场景）。
    pub fn execute_with_tables(&self, tables: &[TableInfo]) -> Result<Vec<std::path::PathBuf>, String> {
        let filtered: Vec<_> = tables
            .iter()
            .filter(|t| self.strategy.is_table_included(&t.name))
            .cloned()
            .collect();

        let engine = TeraEngine::new(
            self.global.clone(),
            self.package.clone(),
            self.strategy.clone(),
        );

        engine.batch_output(&filtered)
    }
}

/// AutoGenerator 构建器。
#[derive(Default)]
pub struct AutoGeneratorBuilder {
    data_source: Option<DataSourceConfig>,
    global: Option<GlobalConfig>,
    package: Option<PackageConfig>,
    strategy: Option<StrategyConfig>,
}

impl AutoGeneratorBuilder {
    pub fn data_source(mut self, config: DataSourceConfig) -> Self {
        self.data_source = Some(config);
        self
    }
    pub fn global(mut self, config: GlobalConfig) -> Self {
        self.global = Some(config);
        self
    }
    pub fn package(mut self, config: PackageConfig) -> Self {
        self.package = Some(config);
        self
    }
    pub fn strategy(mut self, config: StrategyConfig) -> Self {
        self.strategy = Some(config);
        self
    }
    pub fn build(self) -> AutoGenerator {
        AutoGenerator {
            data_source: self.data_source.unwrap_or_default(),
            global: self.global.unwrap_or_default(),
            package: self.package.unwrap_or_default(),
            strategy: self.strategy.unwrap_or_default(),
        }
    }
}
