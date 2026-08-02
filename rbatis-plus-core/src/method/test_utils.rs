//! 测试辅助函数：构造 `TableInfo` 实例供 13 个 method 测试共用。
//!
//! 无 #[cfg(test)] 限制，供集成测试（tests/）和单元测试均可访问。

use crate::derive::{FieldStrategy, IdType};
use crate::metadata::{TableFieldInfo, TableInfo};

/// 构造一个标准的 User 表 `TableInfo`。
///
/// 包含 `id`(pk,auto)、`name`(NotNull)、`big_blob`(Never)、`email`(NotEmpty)
/// —— 覆盖 FieldStrategy 各变体的边界条件。
pub fn user_table_info() -> TableInfo {
    TableInfo {
        entity_type: "User",
        table_name: "users".into(),
        key_column: "id".into(),
        key_property: "id".into(),
        id_type: IdType::Auto,
        field_list: vec![
            TableFieldInfo {
                column: "name".into(),
                property: "name".into(),
                insert_strategy: FieldStrategy::NotNull,
                ..Default::default()
            },
            TableFieldInfo {
                column: "big_blob".into(),
                property: "big_blob".into(),
                insert_strategy: FieldStrategy::Never,
                ..Default::default()
            },
            TableFieldInfo {
                column: "email".into(),
                property: "email".into(),
                insert_strategy: FieldStrategy::NotEmpty,
                ..Default::default()
            },
        ],
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

/// 构造带逻辑删除字段的 Order 表 `TableInfo`。
pub fn order_with_logic_delete() -> TableInfo {
    use crate::metadata::TableFieldInfo;
    use crate::derive::FieldStrategy;

    TableInfo {
        entity_type: "Order",
        table_name: "orders".into(),
        key_column: "id".into(),
        key_property: "id".into(),
        id_type: IdType::Auto,
        field_list: vec![
            TableFieldInfo {
                column: "user_id".into(),
                property: "user_id".into(),
                insert_strategy: FieldStrategy::NotNull,
                ..Default::default()
            },
        ],
        with_logic_delete: true,
        logic_delete_field: Some(TableFieldInfo {
            column: "deleted".into(),
            property: "deleted".into(),
            logic_delete: true,
            logic_not_delete_value: "0".into(),
            logic_delete_value: "1".into(),
            ..Default::default()
        }),
        with_version: true,
        version_field: Some(TableFieldInfo {
            column: "version".into(),
            property: "version".into(),
            version: true,
            ..Default::default()
        }),
        auto_init_result_map: false,
        key_related: false,
        column_format: String::new(),
        under_camel: false,
        result_ordered: false,
        order_by_fields: vec![],
    }
}
