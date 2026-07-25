//! 包名配置（对标 mybatis-plus-generator `PackageConfig`）。

use serde::{Deserialize, Serialize};

/// 包名配置（对标 Java `com.baomidou.mybatisplus.generator.config.PackageConfig`）。
///
/// 控制生成代码的模块和包名结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    /// 父包名，如 `"crate"` 或 `"myapp"`。
    pub parent: String,
    /// 模块名（可选），如 `"user"`。
    pub module_name: Option<String>,
    /// Entity 子包名。
    pub entity: String,
    /// Mapper 子包名。
    pub mapper: String,
    /// Service 子包名。
    pub service: String,
    /// ServiceImpl 子包名。
    pub service_impl: String,
    /// Controller 子包名。
    pub controller: String,
    /// XML 子包名。
    pub xml: String,
}

impl Default for PackageConfig {
    fn default() -> Self {
        Self {
            parent: "crate".to_string(),
            module_name: None,
            entity: "entity".to_string(),
            mapper: "mapper".to_string(),
            service: "service".to_string(),
            service_impl: "service::impl".to_string(),
            controller: "controller".to_string(),
            xml: "mapper::xml".to_string(),
        }
    }
}

impl PackageConfig {
    /// 获取 Entity 包路径（用于 `use` 语句）。
    pub fn entity_package(&self) -> String {
        self.sub_package(&self.entity)
    }

    /// 获取 Mapper 包路径。
    pub fn mapper_package(&self) -> String {
        self.sub_package(&self.mapper)
    }

    /// 获取 Service 包路径。
    pub fn service_package(&self) -> String {
        self.sub_package(&self.service)
    }

    /// 获取 Controller 包路径。
    pub fn controller_package(&self) -> String {
        self.sub_package(&self.controller)
    }

    fn sub_package(&self, sub: &str) -> String {
        match &self.module_name {
            Some(module) => format!("{}::{}::{}", self.parent, module, sub),
            None => format!("{}::{}", self.parent, sub),
        }
    }

    /// 创建构建器。
    pub fn builder() -> PackageConfigBuilder {
        PackageConfigBuilder::default()
    }
}

/// PackageConfig 构建器。
#[derive(Default)]
pub struct PackageConfigBuilder {
    config: PackageConfig,
}

impl PackageConfigBuilder {
    pub fn parent(mut self, parent: impl Into<String>) -> Self {
        self.config.parent = parent.into();
        self
    }
    pub fn module_name(mut self, module: impl Into<String>) -> Self {
        self.config.module_name = Some(module.into());
        self
    }
    pub fn entity(mut self, name: impl Into<String>) -> Self {
        self.config.entity = name.into();
        self
    }
    pub fn mapper(mut self, name: impl Into<String>) -> Self {
        self.config.mapper = name.into();
        self
    }
    pub fn service(mut self, name: impl Into<String>) -> Self {
        self.config.service = name.into();
        self
    }
    pub fn service_impl(mut self, name: impl Into<String>) -> Self {
        self.config.service_impl = name.into();
        self
    }
    pub fn controller(mut self, name: impl Into<String>) -> Self {
        self.config.controller = name.into();
        self
    }
    pub fn build(self) -> PackageConfig {
        self.config
    }
}
