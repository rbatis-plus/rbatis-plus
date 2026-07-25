//! 全局配置（对标 mybatis-plus-generator `GlobalConfig`）。

use serde::{Deserialize, Serialize};

/// 全局生成配置（对标 Java `com.baomidou.mybatisplus.generator.config.GlobalConfig`）。
///
/// 控制输出目录、作者、日期格式等全局设置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// 输出目录（绝对路径），如 `"/home/user/project/src"`。
    pub output_dir: String,
    /// 代码作者名（写入文件头注释）。
    pub author: String,
    /// 是否覆盖已有文件（默认 false）。
    pub file_override: bool,
    /// 日期格式（用于注释中的日期）。
    pub date_format: String,
    /// 是否生成 Swagger 注解（默认 false）。
    pub swagger: bool,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            output_dir: ".".to_string(),
            author: "rbatis-plus-generator".to_string(),
            file_override: false,
            date_format: "%Y-%m-%d".to_string(),
            swagger: false,
        }
    }
}

impl GlobalConfig {
    /// 创建构建器。
    pub fn builder() -> GlobalConfigBuilder {
        GlobalConfigBuilder::default()
    }
}

/// GlobalConfig 构建器。
#[derive(Default)]
pub struct GlobalConfigBuilder {
    config: GlobalConfig,
}

impl GlobalConfigBuilder {
    pub fn output_dir(mut self, dir: impl Into<String>) -> Self {
        self.config.output_dir = dir.into();
        self
    }
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.config.author = author.into();
        self
    }
    pub fn file_override(mut self, val: bool) -> Self {
        self.config.file_override = val;
        self
    }
    pub fn swagger(mut self, val: bool) -> Self {
        self.config.swagger = val;
        self
    }
    pub fn build(self) -> GlobalConfig {
        self.config
    }
}
