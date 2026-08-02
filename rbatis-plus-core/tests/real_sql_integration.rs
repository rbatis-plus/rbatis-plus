//! 集成测试：使用真实 SQLite（内存模式）验证 13 个 method 生成的 SQL 可执行。
//!
//! 对应 Java `mybatis-plus-core/src/test/java/.../injector/` 下的 integration tests。
//! 使用 `rusqlite` 代替 testcontainers，SQLite 内存模式无需 Docker。

use rusqlite::Connection;
use rbatis_plus_core::method::test_utils::user_table_info;
use rbatis_plus_core::method::*;
use rbatis_plus_core::metadata::{TableInfo, TableFieldInfo};
use rbatis_plus_core::derive::{FieldStrategy, IdType};

/// 创建测试用 SQLite 内存表（模拟 user 表）。
fn create_test_table(conn: &Connection) {
    conn.execute_batch(
        "CREATE TABLE users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            big_blob TEXT,
            email TEXT
        )"
    ).expect("Failed to create test table");
}

/// 将 SQL 中所有 `?` 按顺序替换为给定值（数字不加引号，字符串加引号）。
fn with_params(sql: &str, params: &[&str]) -> String {
    let mut result = sql.to_string();
    for param in params {
        let replacement = if param.len() <= 20 && param.parse::<i64>().is_ok() {
            (*param).to_string()
        } else {
            format!("'{}'", param)
        };
        result = result.replacen("?", &replacement, 1);
    }
    result
}

/// 运行单条 SQL 并返回第一列结果（转字符串）。
fn exec_sql(conn: &Connection, sql: &str) -> Result<String, rusqlite::Error> {
    let sql_upper = sql.trim().to_uppercase();
    if sql_upper.starts_with("SELECT") {
        let mut stmt = conn.prepare(sql)?;
        let mut rows = stmt.query_map([], |row| {
            let val: rusqlite::types::Value = row.get(0)?;
            match val {
                rusqlite::types::Value::Integer(i) => Ok(i.to_string()),
                rusqlite::types::Value::Text(t) => Ok(t),
                rusqlite::types::Value::Null => Ok("NULL".to_string()),
                other => Ok(format!("{:?}", other)),
            }
        })?;
        let mut results = Vec::new();
        while let Some(row) = rows.next() {
            results.push(row?.clone());
        }
        Ok(results.join(","))
    } else {
        let affected = conn.execute(sql, [])?;
        Ok(format!("{} rows affected", affected))
    }
}

#[test]
fn test_insert_sql_executable() {
    let conn = Connection::open_in_memory().unwrap();
    create_test_table(&conn);
    let sql = Insert.generate_sql(&user_table_info());
    let executable = with_params(&sql.sql, &["Alice", "alice@example.com"]);
    assert!(exec_sql(&conn, &executable).is_ok());
    assert_eq!(exec_sql(&conn, "SELECT COUNT(*) AS c FROM users").unwrap(), "1");
}

#[test]
fn test_delete_sql_executable() {
    let conn = Connection::open_in_memory().unwrap();
    create_test_table(&conn);
    let sql = Delete.generate_sql(&user_table_info());
    assert!(exec_sql(&conn, &format!("{} WHERE id = 1", sql.sql)).is_ok());
}

#[test]
fn test_delete_by_id_sql_executable() {
    let conn = Connection::open_in_memory().unwrap();
    create_test_table(&conn);
    let sql = DeleteById.generate_sql(&user_table_info());
    assert!(exec_sql(&conn, &with_params(&sql.sql, &["1"])).is_ok());
}

#[test]
fn test_update_sql_executable() {
    let conn = Connection::open_in_memory().unwrap();
    create_test_table(&conn);
    let sql = Update.generate_sql(&user_table_info());
    // Update generates SET with name/email columns, append WHERE
    let full_sql = format!("{} WHERE id = 1", with_params(&sql.sql, &["Bob", "bob@example.com"]));
    assert!(exec_sql(&conn, &full_sql).is_ok());
}

#[test]
fn test_update_by_id_sql_executable() {
    let conn = Connection::open_in_memory().unwrap();
    create_test_table(&conn);
    let sql = UpdateById.generate_sql(&user_table_info());
    // UpdateById generates: UPDATE users SET name = ?, email = ? WHERE id = ?
    // 需要 3 个参数
    let executable = with_params(&sql.sql, &["Charlie", "1", "1"]);
    assert!(exec_sql(&conn, &executable).is_ok());
}

#[test]
fn test_select_by_id_sql_executable() {
    let conn = Connection::open_in_memory().unwrap();
    create_test_table(&conn);
    conn.execute("INSERT INTO users (name, email) VALUES ('Dave', 'dave@example.com')", []).unwrap();
    let sql = SelectById.generate_sql(&user_table_info());
    let result = exec_sql(&conn, &with_params(&sql.sql, &["1"])).unwrap();
    assert!(!result.is_empty(), "Should return a result");
}

#[test]
fn test_select_by_ids_sql_executable() {
    let conn = Connection::open_in_memory().unwrap();
    create_test_table(&conn);
    let sql = SelectByIds.generate_sql(&user_table_info());
    let executable = format!("{} (1, 2)", sql.sql);
    assert!(exec_sql(&conn, &executable).is_ok());
}

#[test]
fn test_select_by_map_sql_executable() {
    let conn = Connection::open_in_memory().unwrap();
    create_test_table(&conn);
    let sql = SelectByMap.generate_sql(&user_table_info());
    assert!(exec_sql(&conn, &sql.sql).is_ok());
}

#[test]
fn test_select_count_sql_executable() {
    let conn = Connection::open_in_memory().unwrap();
    create_test_table(&conn);
    let sql = SelectCount.generate_sql(&user_table_info());
    let result = exec_sql(&conn, &sql.sql).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_select_list_sql_executable() {
    let conn = Connection::open_in_memory().unwrap();
    create_test_table(&conn);
    let sql = SelectList.generate_sql(&user_table_info());
    assert!(exec_sql(&conn, &sql.sql).is_ok());
}

#[test]
fn test_select_maps_sql_executable() {
    let conn = Connection::open_in_memory().unwrap();
    create_test_table(&conn);
    let sql = SelectMaps.generate_sql(&user_table_info());
    assert!(exec_sql(&conn, &sql.sql).is_ok());
}

#[test]
fn test_select_one_sql_executable() {
    let conn = Connection::open_in_memory().unwrap();
    create_test_table(&conn);
    let sql = SelectOne.generate_sql(&user_table_info());
    assert!(exec_sql(&conn, &sql.sql).is_ok());
}

#[test]
fn test_select_objs_sql_executable() {
    let conn = Connection::open_in_memory().unwrap();
    create_test_table(&conn);
    let sql = SelectObjs.generate_sql(&user_table_info());
    assert!(sql.sql.starts_with("SELECT id"));
    assert!(exec_sql(&conn, &sql.sql).is_ok());
}

#[test]
fn test_insert_then_select_roundtrip() {
    let conn = Connection::open_in_memory().unwrap();
    create_test_table(&conn);
    let table = user_table_info();

    // INSERT
    let insert = Insert.generate_sql(&table);
    assert!(exec_sql(&conn, &with_params(&insert.sql, &["Eve", "eve@example.com"])).is_ok());

    // COUNT
    let count = exec_sql(&conn, &SelectCount.generate_sql(&table).sql).unwrap();
    assert_eq!(count, "1");

    // DELETE
    let delete = with_params(&DeleteById.generate_sql(&table).sql, &["1"]);
    assert!(exec_sql(&conn, &delete).is_ok());

    // 验证删除
    let final_count = exec_sql(&conn, &SelectCount.generate_sql(&table).sql).unwrap();
    assert_eq!(final_count, "0");
}

#[test]
fn test_logic_delete_table() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE orders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            version INTEGER DEFAULT 1,
            deleted INTEGER DEFAULT 0
        )"
    ).unwrap();

    let table = TableInfo {
        entity_type: "Order",
        table_name: "orders".into(),
        key_column: "id".into(),
        key_property: "id".into(),
        id_type: IdType::Auto,
        field_list: vec![
            TableFieldInfo { column: "user_id".into(), property: "user_id".into(), insert_strategy: FieldStrategy::NotNull, ..Default::default() },
            TableFieldInfo { column: "version".into(), property: "version".into(), insert_strategy: FieldStrategy::NotNull, version: true, ..Default::default() },
            TableFieldInfo { column: "deleted".into(), property: "deleted".into(), insert_strategy: FieldStrategy::NotNull, logic_delete: true,
                logic_not_delete_value: "0".into(), logic_delete_value: "1".into(), ..Default::default() },
        ],
        with_logic_delete: true,
        logic_delete_field: Some(TableFieldInfo { column: "deleted".into(), property: "deleted".into(), logic_delete: true,
            logic_not_delete_value: "0".into(), logic_delete_value: "1".into(), ..Default::default() }),
        with_version: true,
        version_field: Some(TableFieldInfo { column: "version".into(), property: "version".into(), version: true, ..Default::default() }),
        auto_init_result_map: false,
        key_related: false,
        column_format: String::new(),
        under_camel: false,
        result_ordered: false,
        order_by_fields: vec![],
    };

    // INSERT (user_id + version + deleted)
    let insert = Insert.generate_sql(&table);
    let insert_sql = with_params(&insert.sql, &["1", "1", "0"]);
    assert!(exec_sql(&conn, &insert_sql).is_ok());

    // COUNT
    let count = exec_sql(&conn, &SelectCount.generate_sql(&table).sql).unwrap();
    assert_eq!(count, "1");
}
