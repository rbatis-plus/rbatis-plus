//! 基于 Askama 的模板引擎（对标 mybatis-plus-generator `FreemarkerTemplateEngine`）。
//!
//! 使用 `askama` crate 的 `#[derive(Template)]` 宏，通过 Jinja2 风格的内联模板渲染代码。
//!
//! 对应 Java：`com.baomidou.mybatisplus.generator.engine.FreemarkerTemplateEngine`
//!
//! Askama 特点：
//! - 编译期模板验证，类型安全
//! - 支持 `{% if %}` / `{% for %}` 控制流
//! - `.rs` 扩展名不触发 HTML 转义

use askama::Template;
use rbatis_plus_core::derive::FieldFill;
use rbatis_plus_core::metadata::{TableFieldInfo, TableInfo};

use super::template_engine::TemplateEngine;

// ── 可序列化的模板字段类型 ──

/// Askama 模板用的字段信息。
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct AskamaFieldInfo {
    /// 数据库列名。
    column: String,
    /// Rust 属性名（snake_case）。
    property: String,
    /// Rust 类型名。
    rust_type: String,
    /// 是否主键。
    is_pk: bool,
    /// 是否需要 serde rename（列名与属性名不同）。
    need_serde_rename: bool,
    /// 是否有自动填充。
    has_fill: bool,
    /// 填充策略描述。
    fill_desc: String,
    /// 是否逻辑删除字段。
    is_logic_delete: bool,
    /// 是否版本字段。
    is_version: bool,
}

/// 将 `TableFieldInfo` 转换为 Askama 模板用的字段信息。
fn to_askama_fields(table: &TableInfo, fields: &[TableFieldInfo]) -> Vec<AskamaFieldInfo> {
    fields
        .iter()
        .map(|f| {
            let is_pk = table.have_pk() && f.column == table.key_column;
            let rust_type = infer_rust_type(f);
            AskamaFieldInfo {
                column: f.column.clone(),
                property: f.property.clone(),
                rust_type,
                is_pk,
                need_serde_rename: f.column != f.property,
                has_fill: f.fill != FieldFill::Default,
                fill_desc: format!("{:?}", f.fill),
                is_logic_delete: f.logic_delete,
                is_version: f.version,
            }
        })
        .collect()
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

// ── Askama Entity 模板 ──

/// Askama Entity 模板（Jinja2 风格，`source` 内联）。
///
/// Askama 在编译期验证模板语法，`.rs` 扩展名不触发 HTML 转义。
#[derive(Template)]
#[template(
    source = "//! {{ entity_name }} — 自动生成的 Entity 结构体\n//!\n//! 对应数据库表: `{{ table_name }}`\n//!\n//! 由 rbatis-plus-generator 自动生成，请勿手动修改。\n\nuse serde::{Deserialize, Serialize};\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\n#[derive(rbatis_plus_macros::TableName)]\n#[table_name = \"{{ table_name }}\"]\npub struct {{ entity_name }} {\n{% for field in fields %}\n{% if field.is_pk %}    #[table_id]\n{% endif %}{% if field.need_serde_rename %}    #[serde(rename = \"{{ field.column }}\")]\n{% endif %}    pub {{ field.property }}: {{ field.rust_type }},\n{% endfor %}}\n\nimpl {{ entity_name }} {\n{% for field in fields %}    /// 获取 `{{ field.property }}` 字段引用。\n    pub fn {{ field.property }}(&self) -> &{{ field.rust_type }} {\n        &self.{{ field.property }}\n    }\n{% endfor %}}\n",
    ext = "txt"
)]
struct EntityTemplate<'a> {
    table_name: &'a str,
    entity_name: &'a str,
    fields: &'a [AskamaFieldInfo],
}

// ── Askama Mapper 模板 ──

/// Askama Mapper 模板（Jinja2 风格，`source` 内联）。
#[derive(Template)]
#[template(
    source = "//! {{ entity_name }}Mapper — 自动生成的 Mapper 定义\n//!\n//! 对应数据库表: `{{ table_name }}` 数据访问层\n//!\n//! 由 rbatis-plus-generator 自动生成，请勿手动修改。\n\nuse async_trait::async_trait;\nuse rbatis_plus_core::mapper::BaseMapper;\nuse rbatis_plus_core::conditions::query::QueryWrapper;\nuse rbatis_plus_core::conditions::update::UpdateWrapper;\nuse rbatis_plus_core::page::Page;\nuse rbs::Value;\n\n/// {{ entity_name }} Mapper 类型别名（对标 Java `BaseMapper<{{ entity_name }}>`）。\npub type {{ entity_name }}Mapper = dyn BaseMapper<{{ entity_name }}>;\n\n/// {{ entity_name }} Mapper trait 方法桩（对标 Java `BaseMapper<{{ entity_name }}>` 接口方法）。\n///\n/// 实现此 trait 以提供 {{ entity_name }} 的 CRUD 操作。\n#[async_trait]\npub trait {{ entity_name }}MapperOps: Send + Sync {\n    /// 插入一条记录（对标 Java `BaseMapper.insert(T)`）。\n    async fn insert(&self, entity: &{{ entity_name }}) -> Result<u64, rbatis::Error>;\n\n    /// 根据 ID 删除（对标 Java `BaseMapper.deleteById(Serializable)`）。\n    async fn delete_by_id(&self, id: &Value) -> Result<u64, rbatis::Error>;\n\n    /// 根据条件删除（对标 Java `BaseMapper.delete(Wrapper)`）。\n    async fn delete(&self, wrapper: &QueryWrapper, table_name: &str) -> Result<u64, rbatis::Error>;\n\n    /// 根据 ID 更新（对标 Java `BaseMapper.updateById(T)`）。\n    async fn update_by_id(&self, entity: &{{ entity_name }}) -> Result<u64, rbatis::Error>;\n\n    /// 根据条件更新（对标 Java `BaseMapper.update(T, Wrapper)`）。\n    async fn update(&self, wrapper: &UpdateWrapper, table_name: &str) -> Result<u64, rbatis::Error>;\n\n    /// 根据 ID 查询（对标 Java `BaseMapper.selectById(Serializable)`）。\n    async fn select_by_id(&self, id: &Value) -> Result<Option<{{ entity_name }}>, rbatis::Error>;\n\n    /// 根据条件查询列表（对标 Java `BaseMapper.selectList(Wrapper)`）。\n    async fn select_list(&self, wrapper: &QueryWrapper, table_name: &str) -> Result<Vec<{{ entity_name }}>, rbatis::Error>;\n\n    /// 根据条件查询单条（对标 Java `BaseMapper.selectOne(Wrapper)`）。\n    async fn select_one(&self, wrapper: &QueryWrapper, table_name: &str) -> Result<Option<{{ entity_name }}>, rbatis::Error>;\n\n    /// 根据条件查询总数（对标 Java `BaseMapper.selectCount(Wrapper)`）。\n    async fn select_count(&self, wrapper: &QueryWrapper, table_name: &str) -> Result<u64, rbatis::Error>;\n\n    /// 分页查询（对标 Java `BaseMapper.selectPage(Page, Wrapper)`）。\n    async fn select_page(\n        &self,\n        wrapper: &QueryWrapper,\n        table_name: &str,\n        page_no: u64,\n        page_size: u64,\n    ) -> Result<Page<{{ entity_name }}>, rbatis::Error>;\n}\n",
    ext = "txt"
)]
struct MapperTemplate<'a> {
    table_name: &'a str,
    entity_name: &'a str,
}

/// Askama 模板引擎（对标 Java `FreemarkerTemplateEngine`）。
///
/// 使用 Askama 的编译期模板验证，类型安全且性能优秀。
pub struct AskamaEngine;

impl AskamaEngine {
    /// 创建 Askama 引擎实例。
    pub fn new() -> Self {
        Self
    }
}

impl Default for AskamaEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateEngine for AskamaEngine {
    fn name(&self) -> &str {
        "askama"
    }

    fn render_entity(
        &self,
        table: &TableInfo,
        fields: &[TableFieldInfo],
    ) -> Result<String, rbatis::Error> {
        let entity_name = to_pascal_case(&table.table_name);
        let askama_fields = to_askama_fields(table, fields);

        let template = EntityTemplate {
            table_name: &table.table_name,
            entity_name: &entity_name,
            fields: &askama_fields,
        };

        template
            .render()
            .map_err(|e| rbatis::Error::from(format!("Askama 渲染 Entity 失败: {}", e)))
    }

    fn render_mapper(
        &self,
        table: &TableInfo,
        _fields: &[TableFieldInfo],
    ) -> Result<String, rbatis::Error> {
        let entity_name = to_pascal_case(&table.table_name);

        let template = MapperTemplate {
            table_name: &table.table_name,
            entity_name: &entity_name,
        };

        template
            .render()
            .map_err(|e| rbatis::Error::from(format!("Askama 渲染 Mapper 失败: {}", e)))
    }
}
