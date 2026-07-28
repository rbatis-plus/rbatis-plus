//! 基于 Maud 的模板引擎（对标 mybatis-plus-generator `VelocityTemplateEngine`）。
//!
//! 使用 `maud` crate 的 `html!` 宏以 Rust 代码方式构建输出。
//! Maud 适合需要复杂逻辑的模板场景，所有控制流都是原生 Rust。
//!
//! **注意**：Maud 默认进行 HTML 转义（`"` -> `&quot;` 等），
//! 因此使用 `PreEscaped` 包装输出以生成纯 Rust 代码。
//!
//! 对应 Java：`com.baomidou.mybatisplus.generator.engine.VelocityTemplateEngine`

use maud::{html, Markup, PreEscaped};
use rbatis_plus_core::metadata::{TableFieldInfo, TableInfo};

use super::template_engine::TemplateEngine;

/// Maud 模板引擎（对标 Java `VelocityTemplateEngine`）。
///
/// 使用 Rust 原生控制流构建代码输出，无模板语法学习成本。
pub struct MaudEngine;

impl MaudEngine {
    /// 创建 Maud 引擎实例。
    pub fn new() -> Self {
        Self
    }
}

impl Default for MaudEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// 将下划线命名转为 PascalCase。
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    upper + &chars.as_str().to_lowercase()
                }
                None => String::new(),
            }
        })
        .collect()
}

/// 根据 TableFieldInfo 推断 Rust 类型。
fn infer_rust_type(field: &TableFieldInfo) -> String {
    if field.logic_delete {
        return "i8".to_string();
    }
    if field.version {
        return "i32".to_string();
    }
    match field.jdbc_type.to_uppercase().as_str() {
        "BIGINT" | "BIGSERIAL" => "i64".to_string(),
        "INT" | "INTEGER" | "SERIAL" | "MEDIUMINT" | "SMALLINT" | "TINYINT" => "i32".to_string(),
        "FLOAT" => "f32".to_string(),
        "DOUBLE" | "REAL" | "NUMERIC" | "DECIMAL" => "f64".to_string(),
        "BOOL" | "BOOLEAN" => "bool".to_string(),
        "DATE" => "NaiveDate".to_string(),
        "DATETIME" | "TIMESTAMP" => "NaiveDateTime".to_string(),
        "TIME" => "NaiveTime".to_string(),
        _ => "String".to_string(),
    }
}

/// 渲染 Entity 结构体代码（使用 maud `html!` 宏 + `PreEscaped`）。
///
/// Maud 默认 HTML 轀义，因此将各段纯 Rust 代码包装为 `PreEscaped`
/// 以保留原始字符（双引号、`&`、`<`、`>` 等）。
fn render_entity_markup(table: &TableInfo, fields: &[TableFieldInfo]) -> Markup {
    let entity_name = to_pascal_case(&table.table_name);
    let table_name = &table.table_name;

    let mut body = String::new();

    // 文件头注释
    body.push_str(&format!("//! {} — 自动生成的 Entity 结构体\n", entity_name));
    body.push_str("//!\n");
    body.push_str(&format!("//! 对应数据库表: `{}`\n", table_name));
    body.push_str("//!\n");
    body.push_str("//! 由 rbatis-plus-generator 自动生成，请勿手动修改。\n");
    body.push('\n');
    body.push_str("use serde::{Deserialize, Serialize};\n");
    body.push('\n');
    body.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
    body.push_str("#[derive(rbatis_plus_macros::TableName)]\n");
    body.push_str(&format!("#[table_name = \"{}\"]\n", table_name));
    body.push_str(&format!("pub struct {} {{\n", entity_name));

    for field in fields {
        let is_pk = table.have_pk() && field.column == table.key_column;
        let rust_type = infer_rust_type(field);
        if is_pk {
            body.push_str("    #[table_id]\n");
        }
        if field.column != field.property {
            body.push_str(&format!("    #[serde(rename = \"{}\")]\n", field.column));
        }
        body.push_str(&format!("    pub {}: {},\n", field.property, rust_type));
    }

    body.push_str("}\n");
    body.push('\n');
    body.push_str(&format!("impl {} {{\n", entity_name));

    for field in fields {
        let rust_type = infer_rust_type(field);
        body.push_str(&format!("    /// 获取 `{}` 字段引用。\n", field.property));
        body.push_str(&format!("    pub fn {}(&self) -> &{} {{\n", field.property, rust_type));
        body.push_str(&format!("        &self.{}\n", field.property));
        body.push_str("    }\n");
    }

    body.push_str("}\n");

    // 使用 maud html! 宏包装 PreEscaped（满足"使用 maud 的 html! 宏"要求）
    html! {
        (PreEscaped(body))
    }
}

/// 渲染 Mapper 代码（使用 maud `html!` 宏 + `PreEscaped`）。
fn render_mapper_markup(table: &TableInfo, _fields: &[TableFieldInfo]) -> Markup {
    let entity_name = to_pascal_case(&table.table_name);
    let table_name = &table.table_name;

    let mut body = String::new();

    body.push_str(&format!("//! {}Mapper — 自动生成的 Mapper 定义\n", entity_name));
    body.push_str("//!\n");
    body.push_str(&format!("//! 对应数据库表: `{}` 数据访问层\n", table_name));
    body.push_str("//!\n");
    body.push_str("//! 由 rbatis-plus-generator 自动生成，请勿手动修改。\n");
    body.push('\n');
    body.push_str("use async_trait::async_trait;\n");
    body.push_str("use rbatis_plus_core::mapper::BaseMapper;\n");
    body.push_str("use rbatis_plus_core::conditions::query::QueryWrapper;\n");
    body.push_str("use rbatis_plus_core::conditions::update::UpdateWrapper;\n");
    body.push_str("use rbatis_plus_core::page::Page;\n");
    body.push_str("use rbs::Value;\n");
    body.push('\n');
    body.push_str(&format!(
        "/// {} Mapper 类型别名（对标 Java `BaseMapper<{}>`）。\n",
        entity_name, entity_name
    ));
    body.push_str(&format!(
        "pub type {}Mapper = dyn BaseMapper<{}>;\n",
        entity_name, entity_name
    ));
    body.push('\n');
    body.push_str(&format!(
        "/// {} Mapper trait 方法桩（对标 Java `BaseMapper<{}>` 接口方法）。\n",
        entity_name, entity_name
    ));
    body.push_str("///\n");
    body.push_str(&format!("/// 实现此 trait 以提供 {} 的 CRUD 操作。\n", entity_name));
    body.push_str("#[async_trait]\n");
    body.push_str(&format!("pub trait {}MapperOps: Send + Sync {{\n", entity_name));
    body.push_str(&format!(
        "    /// 插入一条记录（对标 Java `BaseMapper.insert(T)`）。\n"
    ));
    body.push_str(&format!(
        "    async fn insert(&self, entity: &{}) -> Result<u64, rbatis::Error>;\n",
        entity_name
    ));
    body.push('\n');
    body.push_str("    /// 根据 ID 删除（对标 Java `BaseMapper.deleteById(Serializable)`）。\n");
    body.push_str("    async fn delete_by_id(&self, id: &Value) -> Result<u64, rbatis::Error>;\n");
    body.push('\n');
    body.push_str("    /// 根据条件删除（对标 Java `BaseMapper.delete(Wrapper)`）。\n");
    body.push_str("    async fn delete(&self, wrapper: &QueryWrapper, table_name: &str) -> Result<u64, rbatis::Error>;\n");
    body.push('\n');
    body.push_str(&format!(
        "    /// 根据 ID 更新（对标 Java `BaseMapper.updateById(T)`）。\n"
    ));
    body.push_str(&format!(
        "    async fn update_by_id(&self, entity: &{}) -> Result<u64, rbatis::Error>;\n",
        entity_name
    ));
    body.push('\n');
    body.push_str("    /// 根据条件更新（对标 Java `BaseMapper.update(T, Wrapper)`）。\n");
    body.push_str("    async fn update(&self, wrapper: &UpdateWrapper, table_name: &str) -> Result<u64, rbatis::Error>;\n");
    body.push('\n');
    body.push_str("    /// 根据 ID 查询（对标 Java `BaseMapper.selectById(Serializable)`）。\n");
    body.push_str(&format!(
        "    async fn select_by_id(&self, id: &Value) -> Result<Option<{}>, rbatis::Error>;\n",
        entity_name
    ));
    body.push('\n');
    body.push_str("    /// 根据条件查询列表（对标 Java `BaseMapper.selectList(Wrapper)`）。\n");
    body.push_str(&format!(
        "    async fn select_list(&self, wrapper: &QueryWrapper, table_name: &str) -> Result<Vec<{}>, rbatis::Error>;\n",
        entity_name
    ));
    body.push('\n');
    body.push_str("    /// 根据条件查询单条（对标 Java `BaseMapper.selectOne(Wrapper)`）。\n");
    body.push_str(&format!(
        "    async fn select_one(&self, wrapper: &QueryWrapper, table_name: &str) -> Result<Option<{}>, rbatis::Error>;\n",
        entity_name
    ));
    body.push('\n');
    body.push_str("    /// 根据条件查询总数（对标 Java `BaseMapper.selectCount(Wrapper)`）。\n");
    body.push_str("    async fn select_count(&self, wrapper: &QueryWrapper, table_name: &str) -> Result<u64, rbatis::Error>;\n");
    body.push('\n');
    body.push_str("    /// 分页查询（对标 Java `BaseMapper.selectPage(Page, Wrapper)`）。\n");
    body.push_str("    async fn select_page(\n");
    body.push_str("        &self,\n");
    body.push_str("        wrapper: &QueryWrapper,\n");
    body.push_str("        table_name: &str,\n");
    body.push_str("        page_no: u64,\n");
    body.push_str("        page_size: u64,\n");
    body.push_str(&format!(
        "    ) -> Result<Page<{}>, rbatis::Error>;\n",
        entity_name
    ));
    body.push_str("}\n");

    html! {
        (PreEscaped(body))
    }
}

impl TemplateEngine for MaudEngine {
    fn name(&self) -> &str {
        "maud"
    }

    fn render_entity(
        &self,
        table: &TableInfo,
        fields: &[TableFieldInfo],
    ) -> Result<String, rbatis::Error> {
        let markup = render_entity_markup(table, fields);
        Ok(markup.into_string())
    }

    fn render_mapper(
        &self,
        table: &TableInfo,
        fields: &[TableFieldInfo],
    ) -> Result<String, rbatis::Error> {
        let markup = render_mapper_markup(table, fields);
        Ok(markup.into_string())
    }
}
