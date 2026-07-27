//! 代码生成器主入口（对标 Java `com.baomidou.mybatisplus.generator.AutoGenerator`）。
//!
//! 负责协调配置加载、元数据查询、模板渲染、文件输出的完整流程。
//!
//! 对应 Java：
//! - `com.baomidou.mybatisplus.generator.AutoGenerator`（mybatis-plus-generator）
//!
//! 文件来源参考：`mybatis-plus-generator/src/main/java/com/baomidou/mybatisplus/generator/AutoGenerator.java`

use crate::config::{DataSourceConfig, GlobalConfig, PackageConfig, StrategyConfig};
use crate::engine::TeraEngine;
use crate::query::TableInfo;

/// 代码生成器主入口（对标 Java `AutoGenerator`）。
///
/// 负责协调配置加载、元数据查询、模板渲染、文件输出的完整流程。
///
/// ```rust
/// // 示例：使用 AutoGenerator 生成代码
/// use rbatis_plus_generator::{AutoGenerator, DataSourceConfig};
///
/// let generator = AutoGenerator::builder()
///     .data_source(DataSourceConfig::default())
///     .build();
/// // generator.execute().await;
/// ```
pub struct AutoGenerator {
    pub data_source: DataSourceConfig,
    pub global: GlobalConfig,
    pub package: PackageConfig,
    pub strategy: StrategyConfig,
}

impl AutoGenerator {
    /// 创建构建器（对标 `new AutoGenerator()`）。
    pub fn builder() -> super::auto_generator_builder::AutoGeneratorBuilder {
        super::auto_generator_builder::AutoGeneratorBuilder::default()
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
