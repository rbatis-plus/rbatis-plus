//! Vernal 自动配置（对标 mybatis-plus-spring `MybatisPlusAutoConfiguration`）。

use serde::{Deserialize, Serialize};

/// Vernal 自动配置（对标 Java `MybatisPlusProperties`）。
///
/// 控制 RBatis-Plus 的 Web 框架集成行为。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VernalConfig {
    /// 数据库连接 URL。
    pub url: String,
    /// 是否启用分页插件（默认 true）。
    pub enable_pagination: bool,
    /// 是否启用乐观锁插件（默认 false）。
    pub enable_optimistic_locker: bool,
    /// 是否启用多租户插件（默认 false）。
    pub enable_tenant: bool,
    /// 是否启用防全表更新插件（默认 true）。
    pub enable_block_attack: bool,
    /// 默认分页大小（默认 10）。
    pub default_page_size: u64,
    /// 最大分页大小（默认 500）。
    pub max_page_size: u64,
}

impl Default for VernalConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            enable_pagination: true,
            enable_optimistic_locker: false,
            enable_tenant: false,
            enable_block_attack: true,
            default_page_size: 10,
            max_page_size: 500,
        }
    }
}

impl VernalConfig {
    /// 创建构建器。
    pub fn builder() -> VernalConfigBuilder {
        VernalConfigBuilder::default()
    }
}

/// VernalConfig 构建器。
#[derive(Default)]
pub struct VernalConfigBuilder {
    config: VernalConfig,
}

impl VernalConfigBuilder {
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.config.url = url.into();
        self
    }
    pub fn enable_pagination(mut self, val: bool) -> Self {
        self.config.enable_pagination = val;
        self
    }
    pub fn enable_optimistic_locker(mut self, val: bool) -> Self {
        self.config.enable_optimistic_locker = val;
        self
    }
    pub fn enable_tenant(mut self, val: bool) -> Self {
        self.config.enable_tenant = val;
        self
    }
    pub fn enable_block_attack(mut self, val: bool) -> Self {
        self.config.enable_block_attack = val;
        self
    }
    pub fn default_page_size(mut self, size: u64) -> Self {
        self.config.default_page_size = size;
        self
    }
    pub fn max_page_size(mut self, size: u64) -> Self {
        self.config.max_page_size = size;
        self
    }
    pub fn build(self) -> VernalConfig {
        self.config
    }
}
