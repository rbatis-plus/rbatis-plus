//! Integration tests for RBatis-Plus conditions (QueryWrapper / UpdateWrapper).
//!
//! These tests verify SQL generation without a database connection.

use rbatis_plus_core::conditions::query::QueryWrapper;
use rbatis_plus_core::conditions::query::LambdaQueryWrapper;
use rbatis_plus_core::conditions::update::UpdateWrapper;
use rbatis_plus_core::conditions::update::LambdaUpdateWrapper;
use rbatis_plus_core::conditions::{Compare, Func, Join};
use rbatis_plus_core::derive::{TableName, TableId, Version, TableLogic};
use rbatis_plus_generator::*;
use rbatis_plus_generator::config::data_source::DbType;
use rbatis_plus_sqlparser::*;
use rbatis_plus_extension::inner::pagination::PaginationInnerInterceptor;
use rbatis_plus_extension::crypto::{CryptoInnerInterceptor, DefaultEncryptedFieldHandler, EncryptedFieldHandler};
use rbatis_plus_extension::signature::{SignatureInnerInterceptor, DefaultDataSignatureHandler, DataSignatureHandler};
use rbatis_plus_extension::i18n::{DefaultI18nHandler, I18nHandler};
use rbatis_plus_extension::observation::{DefaultObservationHandler, SqlObservationHandler};
use rbatis_plus_extension::insert_ignore::{MysqlInsertIgnoreHandler, PostgreSqlInsertIgnoreHandler, InsertIgnoreHandler};
use rbatis_plus_vernal::*;
use rbatis_plus_vernal::axum_integration::*;

// ---------------------------------------------------------------------------
// QueryWrapper SQL generation tests
// ---------------------------------------------------------------------------

#[test]
fn test_eq_condition() {
    let mut w = QueryWrapper::new();
    w.eq("name", "Alice");
    let sql = w.build_select_sql("users");
    assert!(sql.contains("name = 'Alice'"), "got: {}", sql);
}

#[test]
fn test_ne_condition() {
    let mut w = QueryWrapper::new();
    w.ne("status", 1i64);
    let sql = w.build_select_sql("users");
    assert!(sql.contains("status <> 1"), "got: {}", sql);
}

#[test]
fn test_multiple_conditions_and() {
    let mut w = QueryWrapper::new();
    w.eq("name", "Alice")
        .ge("age", 18i64)
        .lt("age", 65i64);
    let sql = w.build_select_sql("users");
    assert!(sql.contains("name = 'Alice'"), "got: {}", sql);
    assert!(sql.contains("age >= 18"), "got: {}", sql);
    assert!(sql.contains("age < 65"), "got: {}", sql);
}

#[test]
fn test_like_conditions() {
    let mut w = QueryWrapper::new();
    w.like("email", "gmail")
        .like_left("name", "Al")
        .like_right("name", "ce");
    let sql = w.build_select_sql("users");
    assert!(sql.contains("email LIKE '%gmail%'"), "got: {}", sql);
    assert!(sql.contains("name LIKE '%Al'"), "got: {}", sql);
    assert!(sql.contains("name LIKE 'ce%'"), "got: {}", sql);
}

#[test]
fn test_in_condition() {
    let mut w = QueryWrapper::new();
    w.in_values("id", vec![rbs::Value::I64(1), rbs::Value::I64(2), rbs::Value::I64(3)]);
    let sql = w.build_select_sql("users");
    assert!(sql.contains("id IN (1, 2, 3)"), "got: {}", sql);
}

#[test]
fn test_between_condition() {
    let mut w = QueryWrapper::new();
    w.between("age", 18i64, 65i64);
    let sql = w.build_select_sql("users");
    assert!(sql.contains("age BETWEEN 18 AND 65"), "got: {}", sql);
}

#[test]
fn test_is_null_condition() {
    let mut w = QueryWrapper::new();
    w.is_null("deleted_at");
    let sql = w.build_select_sql("users");
    assert!(sql.contains("deleted_at IS NULL"), "got: {}", sql);
}

#[test]
fn test_order_by() {
    let mut w = QueryWrapper::new();
    w.eq("status", 1i64)
        .order_by_desc("create_time")
        .order_by_asc("id");
    let sql = w.build_select_sql("users");
    assert!(sql.contains("ORDER BY create_time DESC, id ASC"), "got: {}", sql);
}

#[test]
fn test_group_by_having() {
    let mut w = QueryWrapper::new();
    w.group_by(&["dept_id", "role"])
        .having("COUNT(*) > 1");
    let sql = w.build_select_sql("users");
    assert!(sql.contains("GROUP BY dept_id, role"), "got: {}", sql);
    assert!(sql.contains("HAVING COUNT(*) > 1"), "got: {}", sql);
}

#[test]
fn test_select_columns() {
    let mut w = QueryWrapper::new();
    w.select(&["id", "name", "email"]);
    let sql = w.build_select_sql("users");
    assert!(sql.starts_with("SELECT id, name, email FROM users"), "got: {}", sql);
}

#[test]
fn test_last_clause() {
    let mut w = QueryWrapper::new();
    w.eq("status", 1i64).last("LIMIT 10");
    let sql = w.build_select_sql("users");
    assert!(sql.ends_with("LIMIT 10"), "got: {}", sql);
}

#[test]
fn test_count_sql() {
    let mut w = QueryWrapper::new();
    w.eq("status", 1i64);
    let sql = w.build_count_sql("users");
    assert!(sql.contains("SELECT COUNT(*) AS total"), "got: {}", sql);
    assert!(sql.contains("WHERE"), "got: {}", sql);
}

// ---------------------------------------------------------------------------
// UpdateWrapper SQL generation tests
// ---------------------------------------------------------------------------

#[test]
fn test_update_set_eq() {
    let mut w = UpdateWrapper::new();
    w.set("name", "Bob").eq("id", 42i64);
    let sql = w.build_update_sql("users");
    assert!(sql.contains("UPDATE users SET"), "got: {}", sql);
    assert!(sql.contains("name = 'Bob'"), "got: {}", sql);
    assert!(sql.contains("WHERE id = 42"), "got: {}", sql);
}

#[test]
fn test_update_set_sql() {
    let mut w = UpdateWrapper::new();
    w.set_sql("age = age + 1").eq("id", 1i64);
    let sql = w.build_update_sql("users");
    assert!(sql.contains("age = age + 1"), "got: {}", sql);
}

#[test]
fn test_update_incr_decr() {
    let mut w = UpdateWrapper::new();
    w.set_incr_by("view_count", 1).eq("id", 1i64);
    let sql = w.build_update_sql("posts");
    assert!(sql.contains("view_count = view_count + 1"), "got: {}", sql);
}

#[test]
fn test_delete_sql() {
    let mut w = UpdateWrapper::new();
    w.eq("status", 0i64).is_null("email");
    let sql = w.build_delete_sql("users");
    assert!(sql.starts_with("DELETE FROM users WHERE"), "got: {}", sql);
}

// ---------------------------------------------------------------------------
// Page / PageRequest
// ---------------------------------------------------------------------------

#[test]
fn test_page_construction() {
    let p = rbatis_plus_core::page::Page::new(
        vec![1, 2, 3],
        100,
        2,
        10,
    );
    assert_eq!(p.records.len(), 3);
    assert_eq!(p.total, 100);
    assert_eq!(p.pages, 10);
    assert!(p.has_next);
}

#[test]
fn test_page_empty() {
    let p = rbatis_plus_core::page::Page::<i32>::empty(1, 10);
    assert!(p.records.is_empty());
    assert_eq!(p.total, 0);
}

#[test]
fn test_page_request_offset() {
    let req = rbatis_plus_core::page::PageRequest::new(3, 20);
    assert_eq!(req.offset(), 40);
}

// ---------------------------------------------------------------------------
// LambdaQueryWrapper tests（类型安全列引用）
// ---------------------------------------------------------------------------

/// 模拟 derive 宏生成的列常量
mod user_columns {
    use rbatis_plus_core::conditions::query::Column;

    pub fn id() -> Column<i64> { Column::new("id") }
    pub fn name() -> Column<String> { Column::new("name") }
    pub fn email() -> Column<String> { Column::new("email") }
    pub fn age() -> Column<i32> { Column::new("age") }
    pub fn status() -> Column<i32> { Column::new("status") }
    pub fn deleted_at() -> Column<String> { Column::new("deleted_at") }
    pub fn create_time() -> Column<String> { Column::new("create_time") }
    pub fn dept_id() -> Column<i64> { Column::new("dept_id") }
    pub fn role() -> Column<String> { Column::new("role") }
}

#[test]
fn test_lambda_eq_condition() {
    let mut w = LambdaQueryWrapper::<()>::new();
    w.eq(user_columns::name(), "Alice");
    let sql = w.build_select_sql("users");
    assert!(sql.contains("name = 'Alice'"), "got: {}", sql);
}

#[test]
fn test_lambda_ne_condition() {
    let mut w = LambdaQueryWrapper::<()>::new();
    w.ne(user_columns::status(), 1i64);
    let sql = w.build_select_sql("users");
    assert!(sql.contains("status <> 1"), "got: {}", sql);
}

#[test]
fn test_lambda_multiple_conditions() {
    let mut w = LambdaQueryWrapper::<()>::new();
    w.eq(user_columns::name(), "Alice")
        .ge(user_columns::age(), 18i32)
        .lt(user_columns::age(), 65i32);
    let sql = w.build_select_sql("users");
    assert!(sql.contains("name = 'Alice'"), "got: {}", sql);
    assert!(sql.contains("age >= 18"), "got: {}", sql);
    assert!(sql.contains("age < 65"), "got: {}", sql);
}

#[test]
fn test_lambda_like_conditions() {
    let mut w = LambdaQueryWrapper::<()>::new();
    w.like(user_columns::email(), "gmail")
        .like_left(user_columns::name(), "Al")
        .like_right(user_columns::name(), "ce");
    let sql = w.build_select_sql("users");
    assert!(sql.contains("email LIKE '%gmail%'"), "got: {}", sql);
    assert!(sql.contains("name LIKE '%Al'"), "got: {}", sql);
    assert!(sql.contains("name LIKE 'ce%'"), "got: {}", sql);
}

#[test]
fn test_lambda_in_condition() {
    let mut w = LambdaQueryWrapper::<()>::new();
    w.in_values(user_columns::id(), vec![rbs::Value::I64(1), rbs::Value::I64(2), rbs::Value::I64(3)]);
    let sql = w.build_select_sql("users");
    assert!(sql.contains("id IN (1, 2, 3)"), "got: {}", sql);
}

#[test]
fn test_lambda_between_condition() {
    let mut w = LambdaQueryWrapper::<()>::new();
    w.between(user_columns::age(), 18i32, 65i32);
    let sql = w.build_select_sql("users");
    assert!(sql.contains("age BETWEEN 18 AND 65"), "got: {}", sql);
}

#[test]
fn test_lambda_is_null_condition() {
    let mut w = LambdaQueryWrapper::<()>::new();
    w.is_null(user_columns::deleted_at());
    let sql = w.build_select_sql("users");
    assert!(sql.contains("deleted_at IS NULL"), "got: {}", sql);
}

#[test]
fn test_lambda_order_by() {
    let mut w = LambdaQueryWrapper::<()>::new();
    w.eq(user_columns::status(), 1i32)
        .order_by_desc(user_columns::create_time())
        .order_by_asc(user_columns::id());
    let sql = w.build_select_sql("users");
    assert!(sql.contains("ORDER BY create_time DESC, id ASC"), "got: {}", sql);
}

#[test]
fn test_lambda_group_by_having() {
    let mut w = LambdaQueryWrapper::<()>::new();
    w.group_by(&[user_columns::dept_id().name(), user_columns::role().name()])
        .having("COUNT(*) > 1");
    let sql = w.build_select_sql("users");
    assert!(sql.contains("GROUP BY dept_id, role"), "got: {}", sql);
    assert!(sql.contains("HAVING COUNT(*) > 1"), "got: {}", sql);
}

#[test]
fn test_lambda_select_columns() {
    let mut w = LambdaQueryWrapper::<()>::new();
    w.select(user_columns::id())
        .select(user_columns::name())
        .select(user_columns::email());
    let sql = w.build_select_sql("users");
    assert!(sql.starts_with("SELECT id, name, email FROM users"), "got: {}", sql);
}

#[test]
fn test_lambda_or_condition() {
    let mut w = LambdaQueryWrapper::<()>::new();
    w.eq(user_columns::status(), 1i32)
        .or()
        .eq(user_columns::status(), 2i32);
    let sql = w.build_select_sql("users");
    assert!(sql.contains("status = 1"), "got: {}", sql);
    assert!(sql.contains("OR"), "got: {}", sql);
    assert!(sql.contains("status = 2"), "got: {}", sql);
}

#[test]
fn test_lambda_last_clause() {
    let mut w = LambdaQueryWrapper::<()>::new();
    w.eq(user_columns::status(), 1i32).last("LIMIT 1");
    let sql = w.build_select_sql("users");
    assert!(sql.ends_with("LIMIT 1"), "got: {}", sql);
}

#[test]
fn test_lambda_count_sql() {
    let mut w = LambdaQueryWrapper::<()>::new();
    w.eq(user_columns::status(), 1i32);
    let sql = w.build_count_sql("users");
    assert!(sql.contains("SELECT COUNT(*) AS total"), "got: {}", sql);
    assert!(sql.contains("WHERE"), "got: {}", sql);
}

#[test]
fn test_lambda_not_in() {
    let mut w = LambdaQueryWrapper::<()>::new();
    w.not_in(user_columns::id(), vec![rbs::Value::I64(5), rbs::Value::I64(6)]);
    let sql = w.build_select_sql("users");
    assert!(sql.contains("id NOT IN (5, 6)"), "got: {}", sql);
}

#[test]
fn test_lambda_in_sql() {
    let mut w = LambdaQueryWrapper::<()>::new();
    w.in_sql(user_columns::dept_id(), "SELECT id FROM departments WHERE active = 1");
    let sql = w.build_select_sql("users");
    assert!(sql.contains("dept_id IN (SELECT id FROM departments WHERE active = 1)"), "got: {}", sql);
}

#[test]
fn test_lambda_not_exists() {
    let mut w = LambdaQueryWrapper::<()>::new();
    w.not_exists("SELECT 1 FROM deleted_users WHERE deleted_users.id = users.id");
    let sql = w.build_select_sql("users");
    assert!(sql.contains("NOT EXISTS (SELECT 1 FROM deleted_users"), "got: {}", sql);
}

#[test]
fn test_lambda_comment_and_first() {
    let mut w = LambdaQueryWrapper::<()>::new();
    w.eq(user_columns::status(), 1i32)
        .comment("/* rbatis-plus */")
        .first("/*+ MAX_EXECUTION_TIME(1000) */");
    let sql = w.build_select_sql("users");
    // first 和 comment 都前置到 SELECT 之前
    assert!(sql.contains("/* rbatis-plus */"), "got: {}", sql);
    assert!(sql.contains("MAX_EXECUTION_TIME(1000)"), "got: {}", sql);
    assert!(sql.contains("SELECT * FROM users WHERE status = 1"), "got: {}", sql);
}

#[test]
fn test_lambda_eq_or_is_null() {
    let mut w = LambdaQueryWrapper::<()>::new();
    w.eq_or_is_null(user_columns::email(), rbs::Value::Null);
    let sql = w.build_select_sql("users");
    assert!(sql.contains("email IS NULL"), "got: {}", sql);

    let mut w2 = LambdaQueryWrapper::<()>::new();
    w2.eq_or_is_null(user_columns::email(), "test@example.com");
    let sql2 = w2.build_select_sql("users");
    assert!(sql2.contains("email = 'test@example.com'"), "got: {}", sql2);
}

// ---------------------------------------------------------------------------
// derive(TableName) 列访问器生成测试
// ---------------------------------------------------------------------------

#[derive(rbatis_plus_macros::TableName)]
#[table_name = "sys_user"]
struct SysUser {
    #[table_id]
    id: i64,
    #[table_field(column = "user_name")]
    name: String,
    email: String,
    age: i32,
}

#[test]
fn test_derive_table_name() {
    assert_eq!(SysUser::table_name(), "sys_user");
}

#[test]
fn test_derive_column_accessors() {
    // column_id() -> Column<i64>，列名为 "id"
    let col_id = SysUser::column_id();
    assert_eq!(col_id.name(), "id");

    // column_name() -> Column<String>，使用 #[table_field(column = "user_name")]
    let col_name = SysUser::column_name();
    assert_eq!(col_name.name(), "user_name");

    // column_email() -> Column<String>，默认列名 = 字段名
    let col_email = SysUser::column_email();
    assert_eq!(col_email.name(), "email");

    // column_age() -> Column<i32>
    let col_age = SysUser::column_age();
    assert_eq!(col_age.name(), "age");
}

#[test]
fn test_derive_column_constants() {
    assert_eq!(SysUser::COLUMN_ID, "id");
    assert_eq!(SysUser::COLUMN_NAME, "user_name");
    assert_eq!(SysUser::COLUMN_EMAIL, "email");
    assert_eq!(SysUser::COLUMN_AGE, "age");
}

#[test]
fn test_derive_lambda_query_full() {
    // 使用 derive 生成的列访问器构建完整查询
    let mut w = LambdaQueryWrapper::<SysUser>::new();
    w.eq(SysUser::column_name(), "Alice")
        .ge(SysUser::column_age(), 18i32)
        .select(SysUser::column_id())
        .select(SysUser::column_name())
        .select(SysUser::column_email())
        .order_by_desc(SysUser::column_id());

    let sql = w.build_select_sql(SysUser::table_name());
    // SELECT 子句
    assert!(sql.contains("SELECT id, user_name, email FROM sys_user"), "got: {}", sql);
    // WHERE 子句（注意列名是 user_name 不是 name）
    assert!(sql.contains("user_name = 'Alice'"), "got: {}", sql);
    assert!(sql.contains("age >= 18"), "got: {}", sql);
    // ORDER BY
    assert!(sql.contains("ORDER BY id DESC"), "got: {}", sql);
}

#[test]
fn test_derive_lambda_with_group_by_constants() {
    // 使用 COLUMN_* 常量（用于 group_by 等需要 &str 的场景）
    let mut w = LambdaQueryWrapper::<SysUser>::new();
    w.group_by(&[SysUser::COLUMN_NAME, SysUser::COLUMN_AGE])
        .having("COUNT(*) > 1");
    let sql = w.build_select_sql(SysUser::table_name());
    assert!(sql.contains("GROUP BY user_name, age"), "got: {}", sql);
    assert!(sql.contains("HAVING COUNT(*) > 1"), "got: {}", sql);
}

// ---------------------------------------------------------------------------
// LambdaUpdateWrapper tests（类型安全列引用 + SET 子句）
// ---------------------------------------------------------------------------

#[test]
fn test_lambda_update_set_eq() {
    let mut w = LambdaUpdateWrapper::<SysUser>::new();
    w.set(SysUser::column_name(), "Bob")
        .eq(SysUser::column_id(), 42i64);
    let sql = w.build_update_sql("sys_user");
    assert!(sql.contains("UPDATE sys_user SET"), "got: {}", sql);
    assert!(sql.contains("user_name = 'Bob'"), "got: {}", sql);
    assert!(sql.contains("WHERE id = 42"), "got: {}", sql);
}

#[test]
fn test_lambda_update_set_incr() {
    let mut w = LambdaUpdateWrapper::<SysUser>::new();
    w.set_incr_by(SysUser::column_age(), 1)
        .eq(SysUser::column_id(), 1i64);
    let sql = w.build_update_sql("sys_user");
    assert!(sql.contains("age = age + 1"), "got: {}", sql);
}

#[test]
fn test_lambda_update_set_decr() {
    let mut w = LambdaUpdateWrapper::<SysUser>::new();
    w.set_decr_by(SysUser::column_age(), 5)
        .eq(SysUser::column_id(), 1i64);
    let sql = w.build_update_sql("sys_user");
    assert!(sql.contains("age = age - 5"), "got: {}", sql);
}

#[test]
fn test_lambda_update_set_sql() {
    let mut w = LambdaUpdateWrapper::<SysUser>::new();
    w.set_sql("update_time = now()")
        .eq(SysUser::column_id(), 1i64);
    let sql = w.build_update_sql("sys_user");
    assert!(sql.contains("update_time = now()"), "got: {}", sql);
}

#[test]
fn test_lambda_update_multiple_set() {
    let mut w = LambdaUpdateWrapper::<SysUser>::new();
    w.set(SysUser::column_name(), "Charlie")
        .set_incr_by(SysUser::column_age(), 1)
        .eq(SysUser::column_id(), 99i64);
    let sql = w.build_update_sql("sys_user");
    assert!(sql.contains("user_name = 'Charlie'"), "got: {}", sql);
    assert!(sql.contains("age = age + 1"), "got: {}", sql);
    assert!(sql.contains("WHERE id = 99"), "got: {}", sql);
}

#[test]
fn test_lambda_update_where_like() {
    let mut w = LambdaUpdateWrapper::<SysUser>::new();
    w.set(SysUser::column_name(), "Updated")
        .like(SysUser::column_email(), "old");
    let sql = w.build_update_sql("sys_user");
    assert!(sql.contains("email LIKE '%old%'"), "got: {}", sql);
}

#[test]
fn test_lambda_update_where_between() {
    let mut w = LambdaUpdateWrapper::<SysUser>::new();
    w.set(SysUser::column_name(), "BulkUpdate")
        .between(SysUser::column_age(), 18i32, 30i32);
    let sql = w.build_update_sql("sys_user");
    assert!(sql.contains("age BETWEEN 18 AND 30"), "got: {}", sql);
}

#[test]
fn test_lambda_update_or_condition() {
    let mut w = LambdaUpdateWrapper::<SysUser>::new();
    w.set(SysUser::column_name(), "Updated")
        .eq(SysUser::column_id(), 1i64)
        .or()
        .eq(SysUser::column_id(), 2i64);
    let sql = w.build_update_sql("sys_user");
    assert!(sql.contains("id = 1"), "got: {}", sql);
    assert!(sql.contains("OR"), "got: {}", sql);
    assert!(sql.contains("id = 2"), "got: {}", sql);
}

#[test]
fn test_lambda_update_delete_sql() {
    let mut w = LambdaUpdateWrapper::<SysUser>::new();
    w.eq(SysUser::column_id(), 42i64);
    let sql = w.build_delete_sql("sys_user");
    assert!(sql.starts_with("DELETE FROM sys_user WHERE"), "got: {}", sql);
    assert!(sql.contains("id = 42"), "got: {}", sql);
}

#[test]
fn test_lambda_update_is_null() {
    let mut w = LambdaUpdateWrapper::<SysUser>::new();
    w.set(SysUser::column_name(), "Updated")
        .is_null(SysUser::column_email());
    let sql = w.build_update_sql("sys_user");
    assert!(sql.contains("email IS NULL"), "got: {}", sql);
}

#[test]
fn test_lambda_update_in_values() {
    let mut w = LambdaUpdateWrapper::<SysUser>::new();
    w.set(SysUser::column_name(), "BatchUpdate")
        .in_values(SysUser::column_id(), vec![rbs::Value::I64(1), rbs::Value::I64(2), rbs::Value::I64(3)]);
    let sql = w.build_update_sql("sys_user");
    assert!(sql.contains("id IN (1, 2, 3)"), "got: {}", sql);
}

#[test]
fn test_lambda_update_not_exists() {
    let mut w = LambdaUpdateWrapper::<SysUser>::new();
    w.set(SysUser::column_name(), "SafeUpdate")
        .not_exists("SELECT 1 FROM locked_users WHERE locked_users.id = sys_user.id");
    let sql = w.build_update_sql("sys_user");
    assert!(sql.contains("NOT EXISTS (SELECT 1 FROM locked_users"), "got: {}", sql);
}

#[test]
fn test_lambda_update_comment_and_first() {
    let mut w = LambdaUpdateWrapper::<SysUser>::new();
    w.set(SysUser::column_name(), "Annotated")
        .eq(SysUser::column_id(), 1i64)
        .comment("/* rbatis-plus update */")
        .first("/*+ MAX_EXECUTION_TIME(5000) */");
    let sql = w.build_update_sql("sys_user");
    assert!(sql.contains("/* rbatis-plus update */"), "got: {}", sql);
    assert!(sql.contains("MAX_EXECUTION_TIME(5000)"), "got: {}", sql);
    assert!(sql.contains("UPDATE sys_user SET"), "got: {}", sql);
}

#[test]
fn test_lambda_update_last_limit() {
    let mut w = LambdaUpdateWrapper::<SysUser>::new();
    w.set(SysUser::column_name(), "Limited")
        .eq(SysUser::column_id(), 1i64)
        .last("LIMIT 1");
    let sql = w.build_update_sql("sys_user");
    assert!(sql.ends_with("LIMIT 1"), "got: {}", sql);
}

// ---------------------------------------------------------------------------
// rbatis-plus-generator 测试（代码生成器）
// ---------------------------------------------------------------------------

#[test]
fn test_generator_config_builder() {
    let ds = DataSourceConfig::builder()
        .url("mysql://root:123456@localhost:3306/test")
        .username("root")
        .password("123456")
        .build();
    assert_eq!(ds.db_type, DbType::MySql);

    let global = GlobalConfig::builder()
        .output_dir("/tmp/generated")
        .author("test")
        .build();
    assert_eq!(global.output_dir, "/tmp/generated");
    assert_eq!(global.author, "test");

    let pkg = PackageConfig::builder()
        .parent("myapp")
        .module_name("user")
        .build();
    assert_eq!(pkg.entity_package(), "myapp::user::entity");

    let strategy = StrategyConfig::builder()
        .include(vec!["sys_user"])
        .table_prefix(vec!["sys_"])
        .build();
    assert!(strategy.is_table_included("sys_user"));
    assert!(!strategy.is_table_included("other_table"));
}

#[test]
fn test_generator_table_info_entity_name() {
    let table = TableInfo {
        name: "sys_user".to_string(),
        comment: "用户表".to_string(),
        primary_keys: vec!["id".to_string()],
        fields: vec![],
    };
    assert_eq!(table.entity_name(), "SysUser");
    assert_eq!(table.module_name(), "sys_user");
}

#[test]
fn test_generator_table_field_property_name() {
    let field = TableField {
        name: "user_name".to_string(),
        comment: "用户名".to_string(),
        db_type: "VARCHAR".to_string(),
        rust_type: "String".to_string(),
        is_primary_key: false,
        is_auto_increment: false,
        is_nullable: true,
    };
    assert_eq!(field.property_name(), "user_name");
}

#[test]
fn test_generator_db_type_detection() {
    assert_eq!(DataSourceConfig::detect_db_type("mysql://localhost/test"), DbType::MySql);
    assert_eq!(DataSourceConfig::detect_db_type("postgres://localhost/test"), DbType::PostgreSql);
    assert_eq!(DataSourceConfig::detect_db_type("sqlite:///tmp/test.db"), DbType::Sqlite);
}

#[test]
fn test_generator_execute_with_tables() {
    use std::fs;
    use tempfile::TempDir;

    let tmp_dir = TempDir::new().unwrap();
    let output_path = tmp_dir.path().to_str().unwrap().to_string();

    let generator = AutoGenerator::builder()
        .global(
            GlobalConfig::builder()
                .output_dir(&output_path)
                .author("test")
                .build()
        )
        .package(
            PackageConfig::builder()
                .parent("myapp")
                .module_name("user")
                .build()
        )
        .strategy(
            StrategyConfig::builder()
                .include(vec!["sys_user"])
                .build()
        )
        .build();

    let table = TableInfo {
        name: "sys_user".to_string(),
        comment: "用户表".to_string(),
        primary_keys: vec!["id".to_string()],
        fields: vec![
            TableField {
                name: "id".to_string(),
                comment: "主键ID".to_string(),
                db_type: "BIGINT".to_string(),
                rust_type: "i64".to_string(),
                is_primary_key: true,
                is_auto_increment: true,
                is_nullable: false,
            },
            TableField {
                name: "user_name".to_string(),
                comment: "用户名".to_string(),
                db_type: "VARCHAR".to_string(),
                rust_type: "String".to_string(),
                is_primary_key: false,
                is_auto_increment: false,
                is_nullable: false,
            },
            TableField {
                name: "email".to_string(),
                comment: "邮箱".to_string(),
                db_type: "VARCHAR".to_string(),
                rust_type: "String".to_string(),
                is_primary_key: false,
                is_auto_increment: false,
                is_nullable: true,
            },
        ],
    };

    let files = generator.execute_with_tables(&[table]).unwrap();
    assert!(files.len() >= 3, "expected at least 3 files (entity/mapper/service), got {}", files.len());

    // 验证 Entity 文件内容
    let entity_path = files.iter().find(|p| p.to_str().unwrap().contains("entity")).unwrap();
    let entity_content = fs::read_to_string(entity_path).unwrap();
    assert!(entity_content.contains("pub struct SysUser"), "entity: {}", entity_content);
    assert!(entity_content.contains("#[table_name = \"sys_user\"]"), "entity: {}", entity_content);
    assert!(entity_content.contains("pub id: i64,"), "entity: {}", entity_content);
    assert!(entity_content.contains("pub user_name: String,"), "entity: {}", entity_content);
    assert!(entity_content.contains("#[table_id]"), "entity: {}", entity_content);
}

// ---------------------------------------------------------------------------
// rbatis-plus-sqlparser 测试（SQL 解析与分页改写）
// ---------------------------------------------------------------------------

#[test]
fn test_sqlparser_parse_select() {
    let parsed = SqlParser::parse("SELECT * FROM users WHERE id = 1");
    assert_eq!(parsed.statement_type, StatementType::Select);
    assert!(!parsed.has_group_by);
    assert!(!parsed.has_order_by);
    assert!(!parsed.has_for_update);
}

#[test]
fn test_sqlparser_parse_insert() {
    let parsed = SqlParser::parse("INSERT INTO users (name) VALUES ('Alice')");
    assert_eq!(parsed.statement_type, StatementType::Insert);
}

#[test]
fn test_sqlparser_parse_update() {
    let parsed = SqlParser::parse("UPDATE users SET name = 'Bob' WHERE id = 1");
    assert_eq!(parsed.statement_type, StatementType::Update);
}

#[test]
fn test_sqlparser_parse_delete() {
    let parsed = SqlParser::parse("DELETE FROM users WHERE id = 1");
    assert_eq!(parsed.statement_type, StatementType::Delete);
}

#[test]
fn test_sqlparser_detect_features() {
    let parsed = SqlParser::parse("SELECT DISTINCT name FROM users GROUP BY name ORDER BY name");
    assert!(parsed.has_distinct);
    assert!(parsed.has_group_by);
    assert!(parsed.has_order_by);
}

#[test]
fn test_sqlparser_for_update() {
    let parsed = SqlParser::parse("SELECT * FROM users WHERE id = 1 FOR UPDATE");
    assert!(parsed.has_for_update);
    assert!(!SqlRewriter::can_paginate(&parsed.original_sql));
}

#[test]
fn test_sqlparser_for_share() {
    let parsed = SqlParser::parse("SELECT * FROM users WHERE id = 1 FOR SHARE");
    assert!(parsed.has_for_update);
    assert!(!SqlRewriter::can_paginate(&parsed.original_sql));
}

#[test]
fn test_sqlparser_can_paginate() {
    assert!(SqlRewriter::can_paginate("SELECT * FROM users"));
    assert!(SqlRewriter::can_paginate("SELECT * FROM users WHERE id = 1"));
    assert!(!SqlRewriter::can_paginate("SELECT * FROM users FOR UPDATE"));
    assert!(!SqlRewriter::can_paginate("INSERT INTO users (name) VALUES ('Alice')"));
}

#[test]
fn test_mysql_dialect_pagination() {
    let dialect = MysqlDialect;
    let sql = dialect.build_pagination_sql("SELECT * FROM users", 0, 10);
    assert_eq!(sql, "SELECT * FROM users LIMIT 0, 10");

    let sql2 = dialect.build_pagination_sql("SELECT * FROM users", 20, 10);
    assert_eq!(sql2, "SELECT * FROM users LIMIT 20, 10");
}

#[test]
fn test_postgresql_dialect_pagination() {
    let dialect = PostgreSqlDialect;
    let sql = dialect.build_pagination_sql("SELECT * FROM users", 0, 10);
    assert_eq!(sql, "SELECT * FROM users LIMIT 10 OFFSET 0");

    let sql2 = dialect.build_pagination_sql("SELECT * FROM users", 20, 10);
    assert_eq!(sql2, "SELECT * FROM users LIMIT 10 OFFSET 20");
}

#[test]
fn test_sqlite_dialect_pagination() {
    let dialect = SqliteDialect;
    let sql = dialect.build_pagination_sql("SELECT * FROM users", 0, 10);
    assert_eq!(sql, "SELECT * FROM users LIMIT 10 OFFSET 0");
}

#[test]
fn test_rewrite_pagination() {
    let dialect = MysqlDialect;
    let sql = SqlRewriter::rewrite_pagination("SELECT * FROM users", 3, 10, &dialect);
    // page 3, size 10 → offset 20
    assert_eq!(sql, "SELECT * FROM users LIMIT 20, 10");
}

#[test]
fn test_rewrite_count_simple() {
    let count_sql = SqlRewriter::rewrite_count("SELECT id, name FROM users WHERE status = 1");
    assert_eq!(count_sql, "SELECT COUNT(*) AS total FROM users WHERE status = 1");
}

#[test]
fn test_rewrite_count_with_group_by() {
    let count_sql = SqlRewriter::rewrite_count("SELECT dept_id, COUNT(*) FROM users GROUP BY dept_id");
    assert!(count_sql.contains("SELECT COUNT(*) AS total FROM"));
    assert!(count_sql.contains("GROUP BY dept_id"));
}

#[test]
fn test_rewrite_count_with_distinct() {
    let count_sql = SqlRewriter::rewrite_count("SELECT DISTINCT name FROM users");
    assert!(count_sql.contains("SELECT COUNT(*) AS total FROM"));
}

#[test]
fn test_replace_select_star() {
    let sql = SqlParser::replace_select_star("SELECT * FROM users", &["id", "name", "email"]);
    assert_eq!(sql, "SELECT id, name, email FROM users");
}

#[test]
fn test_replace_select_star_no_star() {
    let sql = SqlParser::replace_select_star("SELECT id, name FROM users", &["id", "name"]);
    assert_eq!(sql, "SELECT id, name FROM users");
}

#[test]
fn test_dialect_supports() {
    assert!(MysqlDialect.supports("mysql"));
    assert!(MysqlDialect.supports("MySQL"));
    assert!(MysqlDialect.supports("mariadb"));
    assert!(PostgreSqlDialect.supports("postgresql"));
    assert!(PostgreSqlDialect.supports("postgres"));
    assert!(SqliteDialect.supports("sqlite"));
    assert!(!MysqlDialect.supports("postgres"));
}

#[test]
fn test_default_dialects() {
    let dialects = default_dialects();
    assert_eq!(dialects.len(), 3);
}

// ---------------------------------------------------------------------------
// rbatis-plus-vernal 测试（Web 框架集成）
// ---------------------------------------------------------------------------

#[test]
fn test_vernal_config_builder() {
    let config = VernalConfig::builder()
        .url("mysql://root:123456@localhost:3306/test")
        .enable_pagination(true)
        .enable_block_attack(true)
        .default_page_size(20)
        .max_page_size(100)
        .build();
    assert_eq!(config.url, "mysql://root:123456@localhost:3306/test");
    assert!(config.enable_pagination);
    assert!(config.enable_block_attack);
    assert_eq!(config.default_page_size, 20);
    assert_eq!(config.max_page_size, 100);
}

#[test]
fn test_vernal_config_defaults() {
    let config = VernalConfig::default();
    assert!(config.enable_pagination);
    assert!(config.enable_block_attack);
    assert!(!config.enable_optimistic_locker);
    assert!(!config.enable_tenant);
    assert_eq!(config.default_page_size, 10);
    assert_eq!(config.max_page_size, 500);
}

#[test]
fn test_page_param_defaults() {
    let param = PageParam::default();
    assert_eq!(param.page_no, 1);
    assert_eq!(param.page_size, 10);
}

#[test]
fn test_page_param_to_request() {
    let param = PageParam { page_no: 3, page_size: 20 };
    let req = param.to_page_request(500);
    assert_eq!(req.page_no, 3);
    assert_eq!(req.page_size, 20);
}

#[test]
fn test_page_param_max_size_limit() {
    let param = PageParam { page_no: 1, page_size: 9999 };
    let req = param.to_page_request(100);
    assert_eq!(req.page_size, 100); // 被截断到最大值
}

#[test]
fn test_page_param_zero_page_no() {
    let param = PageParam { page_no: 0, page_size: 10 };
    let req = param.to_page_request(500);
    assert_eq!(req.page_no, 1); // 0 被修正为 1
}

#[test]
fn test_page_param_zero_page_size() {
    let param = PageParam { page_no: 1, page_size: 0 };
    let req = param.to_page_request(500);
    assert_eq!(req.page_size, 10); // 0 被修正为默认值 10
}

#[test]
fn test_page_param_empty_page() {
    let param = PageParam { page_no: 2, page_size: 15 };
    let page: rbatis_plus_core::page::Page<i32> = param.empty_page();
    assert!(page.records.is_empty());
    assert_eq!(page.page_no, 2);
    assert_eq!(page.page_size, 15);
}

#[test]
fn test_order_param_no_order() {
    let param = OrderParam { order_by: None, order: None };
    assert!(!param.has_order());
    assert_eq!(param.build_order_by(), "");
}

#[test]
fn test_order_param_asc() {
    let param = OrderParam {
        order_by: Some("create_time".to_string()),
        order: Some("asc".to_string()),
    };
    assert!(param.has_order());
    assert!(param.is_asc());
    assert_eq!(param.build_order_by(), " ORDER BY create_time ASC");
}

#[test]
fn test_order_param_desc() {
    let param = OrderParam {
        order_by: Some("id".to_string()),
        order: Some("desc".to_string()),
    };
    assert!(param.has_order());
    assert!(!param.is_asc());
    assert_eq!(param.build_order_by(), " ORDER BY id DESC");
}

#[test]
fn test_order_param_default_asc() {
    let param = OrderParam {
        order_by: Some("name".to_string()),
        order: None,
    };
    assert!(param.is_asc()); // 默认升序
    assert_eq!(param.build_order_by(), " ORDER BY name ASC");
}

// ---------------------------------------------------------------------------
// PaginationInnerInterceptor 测试（分页拦截器）
// ---------------------------------------------------------------------------

#[test]
fn test_pagination_interceptor_mysql() {
    let interceptor = PaginationInnerInterceptor::new()
        .with_max_limit(500)
        .with_mysql();

    // 测试分页改写
    let sql = interceptor.rewrite_sql("SELECT * FROM users WHERE status = 1", 3, 10);
    assert_eq!(sql, "SELECT * FROM users WHERE status = 1 LIMIT 20, 10");
}

#[test]
fn test_pagination_interceptor_postgresql() {
    let interceptor = PaginationInnerInterceptor::new()
        .with_postgresql();

    let sql = interceptor.rewrite_sql("SELECT * FROM users WHERE status = 1", 3, 10);
    assert_eq!(sql, "SELECT * FROM users WHERE status = 1 LIMIT 10 OFFSET 20");
}

#[test]
fn test_pagination_interceptor_sqlite() {
    let interceptor = PaginationInnerInterceptor::new()
        .with_sqlite();

    let sql = interceptor.rewrite_sql("SELECT * FROM users WHERE status = 1", 3, 10);
    assert_eq!(sql, "SELECT * FROM users WHERE status = 1 LIMIT 10 OFFSET 20");
}

#[test]
fn test_pagination_interceptor_max_limit() {
    let interceptor = PaginationInnerInterceptor::new()
        .with_max_limit(100);

    // 超过限制，截断到 100
    let sql = interceptor.rewrite_sql("SELECT * FROM users", 1, 9999);
    assert_eq!(sql, "SELECT * FROM users LIMIT 0, 100");
}

#[test]
fn test_pagination_interceptor_can_paginate() {
    let interceptor = PaginationInnerInterceptor::new();

    assert!(interceptor.can_paginate("SELECT * FROM users"));
    assert!(interceptor.can_paginate("SELECT id, name FROM users WHERE id > 10"));
    assert!(!interceptor.can_paginate("SELECT * FROM users FOR UPDATE"));
    assert!(!interceptor.can_paginate("INSERT INTO users (name) VALUES ('Alice')"));
    assert!(!interceptor.can_paginate("UPDATE users SET name = 'Bob'"));
    assert!(!interceptor.can_paginate("DELETE FROM users WHERE id = 1"));
}

#[test]
fn test_pagination_interceptor_page_params() {
    let interceptor = PaginationInnerInterceptor::new();

    // 初始无分页参数
    assert!(interceptor.get_page().is_none());

    // 设置分页参数
    interceptor.set_page(3, 20);
    assert_eq!(interceptor.get_page(), Some((3, 20)));

    // 清除分页参数
    interceptor.clear_page();
    assert!(interceptor.get_page().is_none());
}

#[test]
fn test_pagination_interceptor_page_no_correction() {
    let interceptor = PaginationInnerInterceptor::new();

    // page_no = 0 被修正为 1
    interceptor.set_page(0, 10);
    let page = interceptor.get_page().unwrap();
    assert_eq!(page.0, 1); // 被修正为 1
    assert_eq!(page.1, 10);

    // page_size = 0 被修正为 1
    interceptor.set_page(1, 0);
    let page = interceptor.get_page().unwrap();
    assert_eq!(page.0, 1);
    assert_eq!(page.1, 1); // 被修正为 1
}

#[test]
fn test_pagination_interceptor_first_page() {
    let interceptor = PaginationInnerInterceptor::new().with_mysql();

    // 第 1 页，offset = 0
    let sql = interceptor.rewrite_sql("SELECT * FROM users", 1, 10);
    assert_eq!(sql, "SELECT * FROM users LIMIT 0, 10");
}

#[test]
fn test_pagination_interceptor_debug() {
    let interceptor = PaginationInnerInterceptor::new().with_mysql();
    let debug_str = format!("{:?}", interceptor);
    assert!(debug_str.contains("PaginationInnerInterceptor"));
    assert!(debug_str.contains("MySQL"));
}

// ---------------------------------------------------------------------------
// derive 宏扩展测试（TableId / Version / TableLogic / TableField）
// ---------------------------------------------------------------------------

#[derive(rbatis_plus_macros::TableName)]
#[table_name = "sys_order"]
struct SysOrder {
    #[table_id(type = "auto")]
    id: i64,
    #[table_field(column = "order_no")]
    order_no: String,
    #[version]
    version: i32,
    #[table_logic(value = "1", not_value = "0")]
    deleted: i32,
    #[field_fill = "insert"]
    create_time: String,
}

#[test]
fn test_derive_table_id() {
    use rbatis_plus_core::derive::IdType;
    assert_eq!(SysOrder::id_type(), IdType::Auto);
    assert_eq!(SysOrder::id_column(), "id");
}

#[test]
fn test_derive_version() {
    assert_eq!(SysOrder::version_column(), "version");
}

#[test]
fn test_derive_table_logic() {
    assert_eq!(SysOrder::logic_column(), "deleted");
    assert_eq!(SysOrder::logic_value(), "1");
    assert_eq!(SysOrder::not_logic_value(), "0");
}

#[test]
fn test_derive_combined_table_name_and_columns() {
    assert_eq!(SysOrder::table_name(), "sys_order");
    assert_eq!(SysOrder::column_id().name(), "id");
    assert_eq!(SysOrder::column_order_no().name(), "order_no");
    assert_eq!(SysOrder::column_version().name(), "version");
    assert_eq!(SysOrder::column_deleted().name(), "deleted");
    assert_eq!(SysOrder::column_create_time().name(), "create_time");
}

#[test]
fn test_derive_combined_lambda_query() {
    let mut w = LambdaQueryWrapper::<SysOrder>::new();
    w.eq(SysOrder::column_order_no(), "ORD-001")
        .eq(SysOrder::column_deleted(), 0i32);
    let sql = w.build_select_sql(SysOrder::table_name());
    assert!(sql.contains("order_no = 'ORD-001'"), "got: {}", sql);
    assert!(sql.contains("deleted = 0"), "got: {}", sql);
}

// ---------------------------------------------------------------------------
// crypto 模块测试（加密/解密）
// ---------------------------------------------------------------------------

#[test]
fn test_default_encrypted_handler_encrypt_decrypt() {
    let handler = DefaultEncryptedFieldHandler::default();
    let original = "hello world";
    let encrypted = handler.encrypt(original);
    let decrypted = handler.decrypt(&encrypted);
    assert_eq!(decrypted, original);
    assert_ne!(encrypted, original);
}

#[test]
fn test_default_encrypted_handler_custom_key() {
    let handler = DefaultEncryptedFieldHandler::new(b"my-secret-key-123");
    let original = "sensitive data";
    let encrypted = handler.encrypt(original);
    let decrypted = handler.decrypt(&encrypted);
    assert_eq!(decrypted, original);
}

#[test]
fn test_default_encrypted_handler_hmac() {
    let handler = DefaultEncryptedFieldHandler::default();
    let hmac1 = handler.hmac("test value");
    let hmac2 = handler.hmac("test value");
    assert_eq!(hmac1, hmac2);
    assert_ne!(hmac1, handler.hmac("different value"));
}

#[test]
fn test_default_encrypted_handler_verify_hmac() {
    let handler = DefaultEncryptedFieldHandler::default();
    let signature = handler.hmac("my data");
    assert!(handler.verify_hmac("my data", &signature));
    assert!(!handler.verify_hmac("tampered data", &signature));
}

#[test]
fn test_crypto_interceptor_creation() {
    let handler = DefaultEncryptedFieldHandler::default();
    let interceptor = CryptoInnerInterceptor::new(Box::new(handler))
        .with_encrypted_column("name")
        .with_encrypted_column("email");

    assert!(interceptor.is_encrypted("name"));
    assert!(interceptor.is_encrypted("email"));
    assert!(!interceptor.is_encrypted("id"));
}

#[test]
fn test_crypto_interceptor_encrypt_decrypt_value() {
    let handler = DefaultEncryptedFieldHandler::default();
    let interceptor = CryptoInnerInterceptor::new(Box::new(handler))
        .with_encrypted_column("secret");

    let original = "my secret data";
    let encrypted = interceptor.encrypt_value(original);
    let decrypted = interceptor.decrypt_value(&encrypted);
    assert_eq!(decrypted, original);
}

#[test]
fn test_crypto_interceptor_debug() {
    let handler = DefaultEncryptedFieldHandler::default();
    let interceptor = CryptoInnerInterceptor::new(Box::new(handler))
        .with_encrypted_column("name");
    let debug = format!("{:?}", interceptor);
    assert!(debug.contains("CryptoInnerInterceptor"));
    assert!(debug.contains("name"));
}

// ---------------------------------------------------------------------------
// signature 模块测试
// ---------------------------------------------------------------------------

#[test]
fn test_signature_handler_sign_verify() {
    let handler = DefaultDataSignatureHandler::default();
    let data = "id=1&name=Alice&age=30";
    let signature = handler.sign(data);
    assert!(handler.verify(data, &signature));
    assert!(!handler.verify("id=1&name=Bob&age=30", &signature));
}

#[test]
fn test_signature_handler_custom_column() {
    let handler = DefaultDataSignatureHandler::default()
        .with_signature_column("sig");
    assert_eq!(handler.signature_column(), "sig");
}

#[test]
fn test_signature_interceptor_creation() {
    let handler = DefaultDataSignatureHandler::default();
    let interceptor = SignatureInnerInterceptor::new(Box::new(handler));
    assert_eq!(interceptor.handler().signature_column(), "data_signature");
    let sig = interceptor.sign("test data");
    assert!(interceptor.verify("test data", &sig));
}

#[test]
fn test_signature_interceptor_debug() {
    let handler = DefaultDataSignatureHandler::default();
    let interceptor = SignatureInnerInterceptor::new(Box::new(handler));
    let debug = format!("{:?}", interceptor);
    assert!(debug.contains("SignatureInnerInterceptor"));
}

// ---------------------------------------------------------------------------
// i18n 模块测试
// ---------------------------------------------------------------------------

#[test]
fn test_i18n_handler_resolve_column() {
    let handler = DefaultI18nHandler::new("zh_CN");
    assert_eq!(handler.current_locale(), "zh_CN");
    assert_eq!(handler.resolve_column("name", "zh_CN"), "name_zh_CN");
    assert_eq!(handler.resolve_column("name", "en_US"), "name_en_US");
}

#[test]
fn test_i18n_handler_default() {
    let handler = DefaultI18nHandler::default();
    assert_eq!(handler.current_locale(), "zh_CN");
}

// ---------------------------------------------------------------------------
// observation 模块测试
// ---------------------------------------------------------------------------

#[test]
fn test_observation_handler_default() {
    let handler = DefaultObservationHandler::default();
    assert_eq!(handler.slow_query_threshold(), std::time::Duration::from_millis(1000));
}

#[test]
fn test_observation_handler_custom_threshold() {
    let handler = DefaultObservationHandler::new(std::time::Duration::from_millis(500));
    assert_eq!(handler.slow_query_threshold(), std::time::Duration::from_millis(500));
}

#[test]
fn test_observation_handler_on_query() {
    let handler = DefaultObservationHandler::default();
    // 不应 panic
    handler.on_query("SELECT * FROM users", std::time::Duration::from_millis(50));
    handler.on_query("SELECT * FROM orders", std::time::Duration::from_millis(2000));
}

// ---------------------------------------------------------------------------
// insert_ignore 模块测试
// ---------------------------------------------------------------------------

#[test]
fn test_mysql_insert_ignore() {
    let handler = MysqlInsertIgnoreHandler;
    let sql = "INSERT INTO users (name, email) VALUES ('Alice', 'alice@test.com')";
    let rewritten = handler.rewrite(sql);
    assert_eq!(rewritten, "INSERT IGNORE INTO users (name, email) VALUES ('Alice', 'alice@test.com')");
}

#[test]
fn test_mysql_insert_ignore_already_ignored() {
    let handler = MysqlInsertIgnoreHandler;
    let sql = "INSERT IGNORE INTO users (name) VALUES ('Alice')";
    let rewritten = handler.rewrite(sql);
    // 已经是 INSERT IGNORE，不应再加
    assert!(!rewritten.contains("IGNORE IGNORE"));
}

#[test]
fn test_postgresql_insert_ignore() {
    let handler = PostgreSqlInsertIgnoreHandler;
    let sql = "INSERT INTO users (name, email) VALUES ('Alice', 'alice@test.com')";
    let rewritten = handler.rewrite(sql);
    assert_eq!(rewritten, "INSERT INTO users (name, email) VALUES ('Alice', 'alice@test.com') ON CONFLICT DO NOTHING");
}

#[test]
fn test_insert_ignore_non_insert() {
    let handler = MysqlInsertIgnoreHandler;
    let sql = "UPDATE users SET name = 'Bob' WHERE id = 1";
    let rewritten = handler.rewrite(sql);
    assert_eq!(rewritten, sql); // 非 INSERT 语句不改写
}
