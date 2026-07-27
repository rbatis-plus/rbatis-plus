//! AutoGenerator 构建器（对标 Java `AutoGenerator` 的 Builder 静态内部类）。
//!
//! 对应 Java：
//! - `com.baomidou.mybatisplus.generator.AutoGenerator` 中的 Builder 内部类
//!
//! 文件来源参考：`mybatis-plus-generator/src/main/java/com/baomidou/mybatisplus/generator/AutoGenerator.java`

use super::auto_generator::AutoGenerator;
use super::config::{DataSourceConfig, GlobalConfig, PackageConfig, StrategyConfig};

/// AutoGenerator 构建器。
///
/// 流式 API 构建 `AutoGenerator`。
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
