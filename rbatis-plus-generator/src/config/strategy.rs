//! 生成策略配置（对标 mybatis-plus-generator `StrategyConfig`）。

use serde::{Deserialize, Serialize};

/// 生成策略配置（对标 Java `com.baomidou.mybatisplus.generator.config.StrategyConfig`）。
///
/// 控制哪些表生成、命名策略、是否生成各类代码等。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    /// 需要生成的表名列表（为空则生成全部表）。
    pub include: Vec<String>,
    /// 排除的表名列表。
    pub exclude: Vec<String>,
    /// 表前缀（生成 Entity 时去除），如 `"sys_"`。
    pub table_prefix: Vec<String>,
    /// 字段前缀（生成字段时去除）。
    pub field_prefix: Vec<String>,
    /// Entity 命名策略。
    pub entity: EntityStrategy,
    /// 是否生成 Mapper。
    pub generate_mapper: bool,
    /// 是否生成 Service。
    pub generate_service: bool,
    /// 是否生成 Controller。
    pub generate_controller: bool,
    /// 是否生成 XML。
    pub generate_xml: bool,
    /// Entity 是否使用 Lombok 风格（Rust 中对应 derive 宏）。
    pub entity_lombok: bool,
}

/// Entity 生成策略。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityStrategy {
    /// 是否使用 `#[derive(TableName)]` 注解。
    pub use_table_name_annotation: bool,
    /// 是否生成 `#[table_id]` 注解。
    pub use_table_id_annotation: bool,
    /// 是否生成 `#[table_field]` 注解。
    pub use_table_field_annotation: bool,
    /// 是否生成 `#[version]` 注解（乐观锁）。
    pub use_version_annotation: bool,
    /// 是否生成 `#[table_logic]` 注解（逻辑删除）。
    pub use_logic_delete_annotation: bool,
    /// 逻辑删除字段名。
    pub logic_delete_field: Option<String>,
    /// 自动填充字段名列表。
    pub fill_fields: Vec<String>,
}

impl Default for EntityStrategy {
    fn default() -> Self {
        Self {
            use_table_name_annotation: true,
            use_table_id_annotation: true,
            use_table_field_annotation: true,
            use_version_annotation: false,
            use_logic_delete_annotation: false,
            logic_delete_field: None,
            fill_fields: Vec::new(),
        }
    }
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            include: Vec::new(),
            exclude: Vec::new(),
            table_prefix: Vec::new(),
            field_prefix: Vec::new(),
            entity: EntityStrategy::default(),
            generate_mapper: true,
            generate_service: true,
            generate_controller: true,
            generate_xml: true,
            entity_lombok: true,
        }
    }
}

impl StrategyConfig {
    /// 创建构建器。
    pub fn builder() -> StrategyConfigBuilder {
        StrategyConfigBuilder::default()
    }

    /// 检查表名是否应该生成。
    pub fn is_table_included(&self, table_name: &str) -> bool {
        // 排除列表优先
        if self.exclude.iter().any(|e| table_name.contains(e.as_str())) {
            return false;
        }
        // 如果 include 为空，生成全部
        if self.include.is_empty() {
            return true;
        }
        self.include.iter().any(|i| table_name.contains(i.as_str()))
    }

    /// 去除表前缀得到 Entity 名基础部分。
    pub fn remove_table_prefix(&self, table_name: &str) -> String {
        let mut name = table_name.to_string();
        for prefix in &self.table_prefix {
            if name.starts_with(prefix.as_str()) {
                name = name[prefix.len()..].to_string();
                break;
            }
        }
        name
    }
}

/// StrategyConfig 构建器。
#[derive(Default)]
pub struct StrategyConfigBuilder {
    config: StrategyConfig,
}

impl StrategyConfigBuilder {
    pub fn include(mut self, tables: Vec<&str>) -> Self {
        self.config.include = tables.into_iter().map(String::from).collect();
        self
    }
    pub fn exclude(mut self, tables: Vec<&str>) -> Self {
        self.config.exclude = tables.into_iter().map(String::from).collect();
        self
    }
    pub fn table_prefix(mut self, prefixes: Vec<&str>) -> Self {
        self.config.table_prefix = prefixes.into_iter().map(String::from).collect();
        self
    }
    pub fn generate_mapper(mut self, val: bool) -> Self {
        self.config.generate_mapper = val;
        self
    }
    pub fn generate_service(mut self, val: bool) -> Self {
        self.config.generate_service = val;
        self
    }
    pub fn generate_controller(mut self, val: bool) -> Self {
        self.config.generate_controller = val;
        self
    }
    pub fn generate_xml(mut self, val: bool) -> Self {
        self.config.generate_xml = val;
        self
    }
    pub fn entity_lombok(mut self, val: bool) -> Self {
        self.config.entity_lombok = val;
        self
    }
    pub fn build(self) -> StrategyConfig {
        self.config
    }
}
