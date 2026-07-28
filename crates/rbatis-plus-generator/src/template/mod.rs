//! 模板引擎模块（对标 mybatis-plus-generator `engine` 包）。
//!
//! 提供三种模板引擎后端，均实现 `TemplateEngine` trait：
//! - `HandlebarsEngine`：基于 handlebars crate，JSON 上下文驱动
//! - `AskamaEngine`：基于 askama crate，编译期模板验证
//! - `MaudEngine`：基于 maud crate，Rust 原生控制流
//!
//! 对应 Java：`com.baomidou.mybatisplus.generator.engine.AbstractTemplateEngine`

pub mod template_engine;
pub mod handlebars_engine;
pub mod askama_engine;
pub mod maud_engine;

pub use template_engine::TemplateEngine;
pub use handlebars_engine::HandlebarsEngine;
pub use askama_engine::AskamaEngine;
pub use maud_engine::MaudEngine;

#[cfg(test)]
mod tests {
    use super::*;
    use rbatis_plus_core::derive::{FieldStrategy, IdType};
    use rbatis_plus_core::metadata::{TableFieldInfo, TableInfo};

    /// 构建测试用的 TableInfo（对标 Java 测试中的 `TableInfoTest`）。
    ///
    /// `fields` 参数包含所有字段（含主键），与 `render_entity` trait 签名一致。
    fn make_test_table() -> TableInfo {
        TableInfo {
            entity_type: "SysUser",
            table_name: "sys_user".to_string(),
            key_column: "id".to_string(),
            key_property: "id".to_string(),
            id_type: IdType::Auto,
            field_list: vec![],
            with_logic_delete: false,
            logic_delete_field: None,
            with_version: false,
            version_field: None,
            auto_init_result_map: false,
            key_related: false,
            column_format: String::new(),
            under_camel: false,
            result_ordered: false,
            order_by_fields: vec![],
        }
    }

    /// 构建测试用的字段列表（包含主键字段）。
    ///
    /// `render_entity` 的 `fields` 参数应包含所有需要渲染的字段，
    /// 包括主键（通过 `column == table.key_column` 判定）。
    fn make_test_fields() -> Vec<TableFieldInfo> {
        vec![
            TableFieldInfo {
                column: "id".to_string(),
                property: "id".to_string(),
                jdbc_type: "INT".to_string(),
                ..Default::default()
            },
            TableFieldInfo {
                column: "user_name".to_string(),
                property: "user_name".to_string(),
                jdbc_type: "VARCHAR".to_string(),
                insert_strategy: FieldStrategy::NotNull,
                ..Default::default()
            },
            TableFieldInfo {
                column: "age".to_string(),
                property: "age".to_string(),
                jdbc_type: "INT".to_string(),
                insert_strategy: FieldStrategy::NotNull,
                ..Default::default()
            },
            TableFieldInfo {
                column: "email".to_string(),
                property: "email".to_string(),
                jdbc_type: "VARCHAR".to_string(),
                insert_strategy: FieldStrategy::Always,
                ..Default::default()
            },
        ]
    }

    // ── Handlebars 引擎测试 ──

    #[test]
    fn test_handlebars_render_entity() {
        let engine = HandlebarsEngine::new();
        let table = make_test_table();
        let fields = make_test_fields();

        let result = engine.render_entity(&table, &fields);
        assert!(result.is_ok(), "Handlebars Entity 渲染失败: {:?}", result.err());

        let code = result.unwrap();
        // 验证关键结构
        assert!(code.contains("pub struct SysUser"), "缺少结构体定义");
        assert!(code.contains("#[table_name = \"sys_user\"]"), "缺少 table_name 注解");
        assert!(code.contains("#[derive(Debug, Clone, Serialize, Deserialize)]"), "缺少 derive");
        assert!(code.contains("use serde::{Deserialize, Serialize};"), "缺少 use 语句");
        assert!(code.contains("pub id: i32"), "缺少主键字段");
        assert!(code.contains("pub user_name: String"), "缺少 user_name 字段");
        assert!(code.contains("pub age: i32"), "缺少 age 字段");
        assert!(code.contains("pub email: String"), "缺少 email 字段");
        assert!(code.contains("#[table_id]"), "缺少主键注解");
        assert!(code.contains("fn user_name(&self)"), "缺少 getter 方法");
        assert_eq!(engine.name(), "handlebars");
    }

    #[test]
    fn test_handlebars_render_mapper() {
        let engine = HandlebarsEngine::new();
        let table = make_test_table();
        let fields = make_test_fields();

        let result = engine.render_mapper(&table, &fields);
        assert!(result.is_ok(), "Handlebars Mapper 渲染失败: {:?}", result.err());

        let code = result.unwrap();
        assert!(code.contains("pub type SysUserMapper"), "缺少类型别名");
        assert!(code.contains("trait SysUserMapperOps"), "缺少 trait 定义");
        assert!(code.contains("async fn insert"), "缺少 insert 方法");
        assert!(code.contains("async fn select_by_id"), "缺少 select_by_id 方法");
        assert!(code.contains("BaseMapper"), "缺少 BaseMapper 引用");
    }

    // ── Askama 引擎测试 ──

    #[test]
    fn test_askama_render_entity() {
        let engine = AskamaEngine::new();
        let table = make_test_table();
        let fields = make_test_fields();

        let result = engine.render_entity(&table, &fields);
        assert!(result.is_ok(), "Askama Entity 渲染失败: {:?}", result.err());

        let code = result.unwrap();
        assert!(code.contains("pub struct SysUser"), "缺少结构体定义");
        assert!(code.contains("#[table_name = \"sys_user\"]"), "缺少 table_name 注解");
        assert!(code.contains("#[derive(Debug, Clone, Serialize, Deserialize)]"), "缺少 derive");
        assert!(code.contains("use serde::{Deserialize, Serialize};"), "缺少 use 语句");
        assert!(code.contains("pub id: i32"), "缺少主键字段");
        assert!(code.contains("pub user_name: String"), "缺少 user_name 字段");
        assert!(code.contains("pub age: i32"), "缺少 age 字段");
        assert!(code.contains("pub email: String"), "缺少 email 字段");
        assert!(code.contains("#[table_id]"), "缺少主键注解");
        assert!(code.contains("fn user_name(&self)"), "缺少 getter 方法");
        assert_eq!(engine.name(), "askama");
    }

    #[test]
    fn test_askama_render_mapper() {
        let engine = AskamaEngine::new();
        let table = make_test_table();
        let fields = make_test_fields();

        let result = engine.render_mapper(&table, &fields);
        assert!(result.is_ok(), "Askama Mapper 渲染失败: {:?}", result.err());

        let code = result.unwrap();
        assert!(code.contains("pub type SysUserMapper"), "缺少类型别名");
        assert!(code.contains("trait SysUserMapperOps"), "缺少 trait 定义");
        assert!(code.contains("async fn insert"), "缺少 insert 方法");
        assert!(code.contains("async fn select_by_id"), "缺少 select_by_id 方法");
        assert!(code.contains("BaseMapper"), "缺少 BaseMapper 引用");
    }

    // ── Maud 引擎测试 ──

    #[test]
    fn test_maud_render_entity() {
        let engine = MaudEngine::new();
        let table = make_test_table();
        let fields = make_test_fields();

        let result = engine.render_entity(&table, &fields);
        assert!(result.is_ok(), "Maud Entity 渲染失败: {:?}", result.err());

        let code = result.unwrap();
        assert!(code.contains("pub struct SysUser"), "缺少结构体定义");
        assert!(code.contains("#[table_name = \"sys_user\"]"), "缺少 table_name 注解");
        assert!(code.contains("#[derive(Debug, Clone, Serialize, Deserialize)]"), "缺少 derive");
        assert!(code.contains("use serde::{Deserialize, Serialize};"), "缺少 use 语句");
        assert!(code.contains("pub id: i32"), "缺少主键字段");
        assert!(code.contains("pub user_name: String"), "缺少 user_name 字段");
        assert!(code.contains("pub age: i32"), "缺少 age 字段");
        assert!(code.contains("pub email: String"), "缺少 email 字段");
        assert!(code.contains("#[table_id]"), "缺少主键注解");
        assert!(code.contains("fn user_name(&self)"), "缺少 getter 方法");
        assert_eq!(engine.name(), "maud");
    }

    #[test]
    fn test_maud_render_mapper() {
        let engine = MaudEngine::new();
        let table = make_test_table();
        let fields = make_test_fields();

        let result = engine.render_mapper(&table, &fields);
        assert!(result.is_ok(), "Maud Mapper 渲染失败: {:?}", result.err());

        let code = result.unwrap();
        assert!(code.contains("pub type SysUserMapper"), "缺少类型别名");
        assert!(code.contains("trait SysUserMapperOps"), "缺少 trait 定义");
        assert!(code.contains("async fn insert"), "缺少 insert 方法");
        assert!(code.contains("async fn select_by_id"), "缺少 select_by_id 方法");
        assert!(code.contains("BaseMapper"), "缺少 BaseMapper 引用");
    }

    // ── 跨引擎一致性测试 ──

    #[test]
    fn test_all_engines_produce_consistent_structure() {
        let table = make_test_table();
        let fields = make_test_fields();

        let handlebars = HandlebarsEngine::new().render_entity(&table, &fields).unwrap();
        let askama = AskamaEngine::new().render_entity(&table, &fields).unwrap();
        let maud = MaudEngine::new().render_entity(&table, &fields).unwrap();

        // 三个引擎生成的代码应包含相同的关键结构
        for (name, code) in [("handlebars", &handlebars), ("askama", &askama), ("maud", &maud)] {
            assert!(code.contains("pub struct SysUser"), "{}: 缺少结构体", name);
            assert!(code.contains("#[table_name = \"sys_user\"]"), "{}: 缺少注解", name);
            assert!(code.contains("pub id: i32"), "{}: 缺少主键字段", name);
            assert!(code.contains("#[table_id]"), "{}: 缺少 table_id", name);
        }
    }

    #[test]
    fn test_render_with_serde_rename() {
        // 主键列名(order_id) != 属性名(id)，应生成 serde rename
        let table = TableInfo {
            entity_type: "Order",
            table_name: "t_order".to_string(),
            key_column: "order_id".to_string(),
            key_property: "id".to_string(),
            id_type: IdType::AssignId,
            field_list: vec![],
            with_logic_delete: false,
            logic_delete_field: None,
            with_version: false,
            version_field: None,
            auto_init_result_map: false,
            key_related: false,
            column_format: String::new(),
            under_camel: false,
            result_ordered: false,
            order_by_fields: vec![],
        };
        let fields = vec![
            TableFieldInfo {
                column: "order_id".to_string(),
                property: "id".to_string(),
                jdbc_type: "BIGINT".to_string(),
                ..Default::default()
            },
            TableFieldInfo {
                column: "order_no".to_string(),
                property: "order_no".to_string(),
                jdbc_type: "VARCHAR".to_string(),
                ..Default::default()
            },
        ];

        let code = HandlebarsEngine::new().render_entity(&table, &fields).unwrap();
        // 主键列名(order_id) != 属性名(id)，应有 serde rename
        assert!(code.contains("#[serde(rename = \"order_id\")]"), "缺少主键 serde rename");
        assert!(code.contains("pub struct TOrder"), "结构体名应为 PascalCase");
    }

    #[test]
    fn test_render_with_version_and_logic_delete() {
        let table = TableInfo {
            entity_type: "User",
            table_name: "sys_user".to_string(),
            key_column: "id".to_string(),
            key_property: "id".to_string(),
            id_type: IdType::Auto,
            field_list: vec![],
            with_logic_delete: true,
            logic_delete_field: None,
            with_version: true,
            version_field: None,
            auto_init_result_map: false,
            key_related: false,
            column_format: String::new(),
            under_camel: false,
            result_ordered: false,
            order_by_fields: vec![],
        };
        let fields = vec![
            TableFieldInfo {
                column: "id".to_string(),
                property: "id".to_string(),
                jdbc_type: "INT".to_string(),
                ..Default::default()
            },
            TableFieldInfo {
                column: "name".to_string(),
                property: "name".to_string(),
                jdbc_type: "VARCHAR".to_string(),
                ..Default::default()
            },
            TableFieldInfo {
                column: "version".to_string(),
                property: "version".to_string(),
                jdbc_type: "INT".to_string(),
                version: true,
                ..Default::default()
            },
            TableFieldInfo {
                column: "deleted".to_string(),
                property: "deleted".to_string(),
                jdbc_type: "TINYINT".to_string(),
                logic_delete: true,
                ..Default::default()
            },
        ];

        let code = AskamaEngine::new().render_entity(&table, &fields).unwrap();
        // version 字段应推断为 i32
        assert!(code.contains("pub version: i32"), "version 字段类型应为 i32");
        // logic_delete 字段应推断为 i8
        assert!(code.contains("pub deleted: i8"), "deleted 字段类型应为 i8");
    }
}
