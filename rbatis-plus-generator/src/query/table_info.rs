//! 数据库表元数据（对标 mybatis-plus-generator `TableInfo` + `TableField`）。
//!
//! 从 JDBC/数据库元数据查询结果映射而来，供模板引擎渲染使用。

use serde::{Deserialize, Serialize};

/// 数据库表信息（对标 Java `com.baomidou.mybatisplus.generator.metadata.TableInfo`）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    /// 表名，如 `"sys_user"`。
    pub name: String,
    /// 表注释。
    pub comment: String,
    /// 主键列名列表。
    pub primary_keys: Vec<String>,
    /// 字段列表。
    pub fields: Vec<TableField>,
}

/// 数据库列信息（对标 Java `com.baomidou.mybatisplus.generator.metadata.TableField`）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableField {
    /// 数据库列名，如 `"user_name"`。
    pub name: String,
    /// 列注释。
    pub comment: String,
    /// 数据库类型名，如 `"VARCHAR"`、`"BIGINT"`。
    pub db_type: String,
    /// 映射后的 Rust 类型名，如 `"String"`、`"i64"`。
    pub rust_type: String,
    /// 是否主键。
    pub is_primary_key: bool,
    /// 是否自增。
    pub is_auto_increment: bool,
    /// 是否可为空。
    pub is_nullable: bool,
}

impl TableInfo {
    /// 将表名转为 PascalCase 结构体名。
    ///
    /// 对应 Java `NameConverter` + `EntityNameConverter`
    pub fn entity_name(&self) -> String {
        use inflector::Inflector;
        self.name.to_snake_case().to_pascal_case()
    }

    /// 将表名转为 snake_case 模块名。
    pub fn module_name(&self) -> String {
        use inflector::Inflector;
        self.name.to_snake_case()
    }
}

impl TableField {
    /// 将列名转为 snake_case Rust 字段名。
    ///
    /// 对应 Java `INameConvert.propertyName()`
    pub fn property_name(&self) -> String {
        use inflector::Inflector;
        self.name.to_snake_case()
    }
}
