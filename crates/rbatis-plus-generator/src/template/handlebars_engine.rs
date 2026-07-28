//! 基于 Handlebars 的模板引擎（对标 mybatis-plus-generator `FreemarkerTemplateEngine`）。
//!
//! 使用 `handlebars` crate 注册内联模板，通过 JSON 上下文渲染 Entity 和 Mapper 代码。
//!
//! 对应 Java：`com.baomidou.mybatisplus.generator.engine.FreemarkerTemplateEngine`

use handlebars::Handlebars;
use rbatis_plus_core::derive::FieldFill;
use rbatis_plus_core::metadata::{TableFieldInfo, TableInfo};
use serde::Serialize;

use super::template_engine::TemplateEngine;

// ── 可序列化的模板上下文类型（用于 Handlebars JSON 序列化） ──

/// 模板渲染用的表信息（对标 Java `TemplateConfig` 上下文变量）。
#[derive(Debug, Clone, Serialize)]
struct TemplateTableInfo {
    /// 表名，如 `"sys_user"`。
    table_name: String,
    /// Entity 结构体名（PascalCase），如 `"SysUser"`。
    entity_name: String,
    /// 表注释。
    comment: String,
    /// 是否有主键。
    have_pk: bool,
    /// 主键列名。
    key_column: String,
    /// 主键属性名。
    key_property: String,
    /// 是否有逻辑删除字段。
    with_logic_delete: bool,
    /// 是否有版本字段。
    with_version: bool,
    /// 字段列表。
    fields: Vec<TemplateFieldInfo>,
}

/// 模板渲染用的字段信息。
#[derive(Debug, Clone, Serialize)]
struct TemplateFieldInfo {
    /// 数据库列名。
    column: String,
    /// Rust 属性名（snake_case）。
    property: String,
    /// Rust 类型名，如 `"String"`、`"i64"`。
    rust_type: String,
    /// 是否主键。
    is_pk: bool,
    /// 字段注释。
    comment: String,
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

/// 将 `TableInfo` + `TableFieldInfo` 转换为模板上下文。
fn to_template_context(
    table: &TableInfo,
    fields: &[TableFieldInfo],
) -> TemplateTableInfo {
    let entity_name = to_pascal_case(&table.table_name);

    let template_fields: Vec<TemplateFieldInfo> = fields
        .iter()
        .enumerate()
        .map(|(_i, f)| {
            let is_pk = table.have_pk() && f.column == table.key_column;
            let rust_type = infer_rust_type(f);
            TemplateFieldInfo {
                column: f.column.clone(),
                property: f.property.clone(),
                rust_type,
                is_pk,
                comment: format!("/// 字段 `{}`", f.column),
                need_serde_rename: f.column != f.property,
                has_fill: f.fill != FieldFill::Default,
                fill_desc: format!("{:?}", f.fill),
                is_logic_delete: f.logic_delete,
                is_version: f.version,
            }
        })
        .collect();

    TemplateTableInfo {
        table_name: table.table_name.clone(),
        entity_name,
        comment: format!("对应数据库表: `{}`", table.table_name),
        have_pk: table.have_pk(),
        key_column: table.key_column.clone(),
        key_property: table.key_property.clone(),
        with_logic_delete: table.with_logic_delete,
        with_version: table.with_version,
        fields: template_fields,
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

/// 根据 TableFieldInfo 推断 Rust 类型（简化版本，实际应基于 jdbc_type 映射）。
fn infer_rust_type(field: &TableFieldInfo) -> String {
    // 如果有 logic_delete 字段，使用 i8
    if field.logic_delete {
        return "i8".to_string();
    }
    if field.version {
        return "i32".to_string();
    }
    // 基于 jdbc_type 的简化映射
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

/// Handlebars 模板引擎（对标 Java `FreemarkerTemplateEngine`）。
///
/// 使用 `handlebars::Handlebars` 注册内联模板，通过 JSON 上下文渲染代码。
pub struct HandlebarsEngine;

impl HandlebarsEngine {
    /// 创建 Handlebars 引擎实例。
    pub fn new() -> Self {
        Self
    }
}

impl Default for HandlebarsEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Entity 模板（Handlebars 语法）。
///
/// 注意：Rust 代码中的 `{{` / `}}` 必须转义为 `{{{{` / `}}}}`。
const ENTITY_TEMPLATE: &str = r#"//! {{entity_name}} — 自动生成的 Entity 结构体
//!
//! {{comment}}
//!
//! 由 rbatis-plus-generator 自动生成，请勿手动修改。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(rbatis_plus_macros::TableName)]
#[table_name = "{{table_name}}"]
pub struct {{entity_name}} {
{{#each fields}}
    {{#if this.is_pk}}#[table_id]
    {{/if}}{{#if this.need_serde_rename}}#[serde(rename = "{{this.column}}")]
    {{/if}}pub {{this.property}}: {{this.rust_type}},
{{/each}}
}

impl {{entity_name}} {
{{#each fields}}
    /// 获取 `{{this.property}}` 字段引用。
    pub fn {{this.property}}(&self) -> &{{this.rust_type}} {
        &self.{{this.property}}
    }
{{/each}}
}
"#;

/// Mapper 模板（Handlebars 语法）。
const MAPPER_TEMPLATE: &str = r#"//! {{entity_name}}Mapper — 自动生成的 Mapper 定义
//!
//! {{comment}} 数据访问层
//!
//! 由 rbatis-plus-generator 自动生成，请勿手动修改。

use async_trait::async_trait;
use rbatis_plus_core::mapper::BaseMapper;
use rbatis_plus_core::conditions::query::QueryWrapper;
use rbatis_plus_core::conditions::update::UpdateWrapper;
use rbatis_plus_core::page::Page;
use rbs::Value;

/// {{entity_name}} Mapper 类型别名（对标 Java `BaseMapper<{{entity_name}}>`）。
pub type {{entity_name}}Mapper = dyn BaseMapper<{{entity_name}}>;

/// {{entity_name}} Mapper trait 方法桩（对标 Java `BaseMapper<{{entity_name}}>` 接口方法）。
///
/// 实现此 trait 以提供 {{entity_name}} 的 CRUD 操作。
#[async_trait]
pub trait {{entity_name}}MapperOps: Send + Sync {
    /// 插入一条记录（对标 Java `BaseMapper.insert(T)`）。
    async fn insert(&self, entity: &{{entity_name}}) -> Result<u64, rbatis::Error>;

    /// 根据 ID 删除（对标 Java `BaseMapper.deleteById(Serializable)`）。
    async fn delete_by_id(&self, id: &Value) -> Result<u64, rbatis::Error>;

    /// 根据条件删除（对标 Java `BaseMapper.delete(Wrapper)`）。
    async fn delete(&self, wrapper: &QueryWrapper, table_name: &str) -> Result<u64, rbatis::Error>;

    /// 根据 ID 更新（对标 Java `BaseMapper.updateById(T)`）。
    async fn update_by_id(&self, entity: &{{entity_name}}) -> Result<u64, rbatis::Error>;

    /// 根据条件更新（对标 Java `BaseMapper.update(T, Wrapper)`）。
    async fn update(&self, wrapper: &UpdateWrapper, table_name: &str) -> Result<u64, rbatis::Error>;

    /// 根据 ID 查询（对标 Java `BaseMapper.selectById(Serializable)`）。
    async fn select_by_id(&self, id: &Value) -> Result<Option<{{entity_name}}>, rbatis::Error>;

    /// 根据条件查询列表（对标 Java `BaseMapper.selectList(Wrapper)`）。
    async fn select_list(&self, wrapper: &QueryWrapper, table_name: &str) -> Result<Vec<{{entity_name}}>, rbatis::Error>;

    /// 根据条件查询单条（对标 Java `BaseMapper.selectOne(Wrapper)`）。
    async fn select_one(&self, wrapper: &QueryWrapper, table_name: &str) -> Result<Option<{{entity_name}}>, rbatis::Error>;

    /// 根据条件查询总数（对标 Java `BaseMapper.selectCount(Wrapper)`）。
    async fn select_count(&self, wrapper: &QueryWrapper, table_name: &str) -> Result<u64, rbatis::Error>;

    /// 分页查询（对标 Java `BaseMapper.selectPage(Page, Wrapper)`）。
    async fn select_page(
        &self,
        wrapper: &QueryWrapper,
        table_name: &str,
        page_no: u64,
        page_size: u64,
    ) -> Result<Page<{{entity_name}}>, rbatis::Error>;
}
"#;

impl TemplateEngine for HandlebarsEngine {
    fn name(&self) -> &str {
        "handlebars"
    }

    fn render_entity(
        &self,
        table: &TableInfo,
        fields: &[TableFieldInfo],
    ) -> Result<String, rbatis::Error> {
        let reg = Handlebars::new();
        let ctx = to_template_context(table, fields);
        reg.render_template(ENTITY_TEMPLATE, &ctx)
            .map_err(|e| rbatis::Error::from(format!("Handlebars 渲染 Entity 失败: {}", e)))
    }

    fn render_mapper(
        &self,
        table: &TableInfo,
        fields: &[TableFieldInfo],
    ) -> Result<String, rbatis::Error> {
        let reg = Handlebars::new();
        let ctx = to_template_context(table, fields);
        // Mapper 模板中也需要 entity_name 和 comment
        let mut mapper_ctx = serde_json::to_value(&ctx)
            .map_err(|e| rbatis::Error::from(format!("序列化上下文失败: {}", e)))?;
        // 注入 entity_name 到顶层（Handlebars 需要）
        if let Some(obj) = mapper_ctx.as_object_mut() {
            obj.insert(
                "entity_name".to_string(),
                serde_json::Value::String(ctx.entity_name.clone()),
            );
        }
        reg.render_template(MAPPER_TEMPLATE, &mapper_ctx)
            .map_err(|e| rbatis::Error::from(format!("Handlebars 渲染 Mapper 失败: {}", e)))
    }
}
