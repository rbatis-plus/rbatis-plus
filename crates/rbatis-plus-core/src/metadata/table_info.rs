// Source: mybatis-plus-core/.../metadata/TableInfo.java
// Source: mybatis-plus-core/.../metadata/TableFieldInfo.java

use crate::derive::{
    FieldFill, FieldStrategy, IdType, TableFieldAttr,
};

/// Per-entity table metadata.
///
/// Mirrors Java `com.baomidou.mybatisplus.core.metadata.TableInfo`.
#[derive(Debug, Clone)]
pub struct TableInfo {
    /// The entity type name (Rust struct name).
    pub entity_type: &'static str,
    /// The database table name.
    pub table_name: String,
    /// Primary key column name.
    pub key_column: String,
    /// Primary key property (Rust field) name.
    pub key_property: String,
    /// Primary key generation strategy.
    pub id_type: IdType,
    /// All field infos (excluding the PK).
    pub field_list: Vec<TableFieldInfo>,
    /// Whether this entity has a logic-delete field.
    pub with_logic_delete: bool,
    /// The logic-delete field info (if any).
    pub logic_delete_field: Option<TableFieldInfo>,
    /// Whether this entity has a version field.
    pub with_version: bool,
    /// The version field info (if any).
    pub version_field: Option<TableFieldInfo>,
}

impl TableInfo {
    /// Build the SELECT column list (all fields, comma-separated).
    pub fn all_sql_select(&self) -> String {
        let mut cols = vec![self.key_column.clone()];
        for f in &self.field_list {
            if f.select {
                cols.push(f.column.clone());
            }
        }
        cols.join(", ")
    }

    /// Whether the entity has a primary key.
    pub fn have_pk(&self) -> bool {
        !self.key_column.is_empty()
    }

    /// Build the INSERT column list (all non-null fields).
    pub fn all_insert_sql_column(&self, prefix: &str) -> String {
        let _ = prefix;
        let mut cols = vec![self.key_column.clone()];
        for f in &self.field_list {
            if f.insert_strategy != FieldStrategy::Never {
                cols.push(f.column.clone());
            }
        }
        cols.join(", ")
    }

    /// Build the WHERE PK fragment: `key_column = #{prefix.key_property}`.
    pub fn get_sql_where(&self, prefix: &str) -> String {
        format!("{} = {{{}{}}}", self.key_column, prefix, self.key_property)
    }
}

/// Per-field metadata.
///
/// Mirrors Java `com.baomidou.mybatisplus.core.metadata.TableFieldInfo`.
#[derive(Debug, Clone)]
pub struct TableFieldInfo {
    /// The database column name.
    pub column: String,
    /// The Rust property (field) name.
    pub property: String,
    /// The el expression (for XML-style mapping, usually empty in Rust).
    pub el: String,
    /// Insert strategy.
    pub insert_strategy: FieldStrategy,
    /// Update strategy.
    pub update_strategy: FieldStrategy,
    /// Where strategy.
    pub where_strategy: FieldStrategy,
    /// Auto-fill behaviour.
    pub fill: FieldFill,
    /// Whether to include this field in SELECT.
    pub select: bool,
    /// Whether this is the version field (optimistic lock).
    pub version: bool,
    /// Whether this is the logic-delete field.
    pub logic_delete: bool,
    /// The value representing "not deleted".
    pub logic_not_delete_value: String,
    /// The value representing "deleted".
    pub logic_delete_value: String,
    /// Raw SET expression for update (e.g. `"now()"` or `"%s+1"`).
    pub update: String,
}

impl TableFieldInfo {
    /// Create from a `TableFieldAttr` (annotation descriptor).
    pub fn from_attr(attr: &TableFieldAttr) -> Self {
        Self {
            column: attr.column.to_string(),
            property: attr.property.to_string(),
            el: String::new(),
            insert_strategy: attr.insert_strategy,
            update_strategy: attr.update_strategy,
            where_strategy: attr.where_strategy,
            fill: attr.fill,
            select: attr.select,
            version: attr.version,
            logic_delete: attr.logic_delete,
            logic_not_delete_value: attr.logic_not_delete_value.to_string(),
            logic_delete_value: attr.logic_delete_value.to_string(),
            update: attr.update.to_string(),
        }
    }
}

impl Default for TableFieldInfo {
    fn default() -> Self {
        Self {
            column: String::new(),
            property: String::new(),
            el: String::new(),
            insert_strategy: FieldStrategy::default(),
            update_strategy: FieldStrategy::default(),
            where_strategy: FieldStrategy::default(),
            fill: FieldFill::default(),
            select: true,
            version: false,
            logic_delete: false,
            logic_not_delete_value: String::new(),
            logic_delete_value: String::new(),
            update: String::new(),
        }
    }
}
