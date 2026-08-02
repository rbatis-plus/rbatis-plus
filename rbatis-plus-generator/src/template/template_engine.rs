//! 模板引擎 trait 定义（对标 mybatis-plus-generator `engine/TemplateEngine`）。
//!
//! 所有模板引擎后端（Handlebars、Askama、Maud）均实现此 trait，
//! 提供统一的 Entity 和 Mapper 代码渲染接口。
//!
//! 对应 Java：`com.baomidou.mybatisplus.generator.engine.AbstractTemplateEngine`

use rbatis_plus_core::metadata::{TableFieldInfo, TableInfo};

/// 模板引擎 trait（对标 Java `AbstractTemplateEngine`）。
///
/// 每个引擎后端负责将表元数据（`TableInfo` + `Vec<TableFieldInfo>`）
/// 渲染为 Rust 源代码字符串。
///
/// # 实现要求
///
/// - `name()` 返回引擎名称（如 `"handlebars"`、`"askama"`、`"maud"`）
/// - `render_entity()` 生成 Entity 结构体代码
/// - `render_mapper()` 生成 Mapper trait 实现代码
pub trait TemplateEngine: Send + Sync {
    /// 引擎名称（对标 Java `AbstractTemplateEngine.engineName()`）。
    fn name(&self) -> &str;

    /// 渲染 Entity 结构体代码（对标 `entity.java.vm`）。
    ///
    /// 生成内容包括：模块文档注释、`use` 语句、`#[derive(...)]` 属性、
    /// `#[table_name]` 注解、结构体字段定义、getter 方法实现。
    fn render_entity(
        &self,
        table: &TableInfo,
        fields: &[TableFieldInfo],
    ) -> Result<String, rbatis::Error>;

    /// 渲染 Mapper 代码（对标 `mapper.java.vm`）。
    ///
    /// 生成内容包括：模块文档注释、`use` 语句、`BaseMapper<T>` 类型别名、
    /// trait 实现方法桩代码。
    fn render_mapper(
        &self,
        table: &TableInfo,
        fields: &[TableFieldInfo],
    ) -> Result<String, rbatis::Error>;
}
