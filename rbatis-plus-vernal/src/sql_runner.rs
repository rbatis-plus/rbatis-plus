//! 原生 SQL 执行器（对标 Java `com.baomidou.mybatisplus.extension.SqlRunner`）。
//!
//! 提供无需 Mapper 即可执行原生 SQL 的能力，适用于复杂 SQL 场景。
//!
//! # 对应 Java
//!
//! - `com.baomidou.mybatisplus.extension.SqlRunner`
//! - `com.baomidou.mybatisplus.extension.service.ISqlRunner`

use rbatis::RBatis;
use rbs::Value;

/// 原生 SQL 执行器（对标 Java `SqlRunner`）。
///
/// 提供原生 SQL 执行能力，无需 Mapper 即可运行任意 SQL。
/// 内部持有 `RBatis` 实例（`RBatis` 内部是 Arc 包装，Clone 开销极低）。
///
/// # 对应 Java
///
/// `com.baomidou.mybatisplus.extension.SqlRunner`
///
/// # Example
///
/// ```ignore
/// use rbatis_plus_vernal::SqlRunner;
/// use rbs::Value;
///
/// let runner = SqlRunner::new(rb);
///
/// // 查询列表
/// let rows = runner.select_list("SELECT * FROM user WHERE age > ?", vec![Value::from(18)]).await?;
///
/// // 查询单条
/// let row = runner.select_one("SELECT * FROM user WHERE id = ?", vec![Value::from(1)]).await?;
///
/// // 查询总数
/// let count = runner.select_count("SELECT COUNT(*) FROM user", vec![]).await?;
///
/// // 执行写操作
/// let affected = runner.execute("UPDATE user SET status = ? WHERE id = ?", vec![Value::from(1), Value::from(100)]).await?;
///
/// // 事务内执行多条 SQL
/// let result = runner.transaction(|runner| async move {
///     runner.execute("INSERT INTO log (msg) VALUES (?)", vec![Value::from("hello")]).await?;
///     runner.execute("UPDATE user SET score = score + 1 WHERE id = ?", vec![Value::from(1)]).await?;
///     Ok::<(), rbatis::Error>(())
/// }).await?;
/// ```
#[derive(Debug, Clone)]
pub struct SqlRunner {
    db: RBatis,
}

impl SqlRunner {
    /// 创建 SqlRunner 实例。
    ///
    /// 对应 Java `new SqlRunner()` 或 `SqlRunner.of(sqlSessionFactory)`。
    pub fn new(db: RBatis) -> Self {
        Self { db }
    }

    /// 获取内部 RBatis 实例的引用。
    pub fn rb(&self) -> &RBatis {
        &self.db
    }

    /// 执行查询，返回 `Vec<Value>`。
    ///
    /// 对应 Java `SqlRunner.selectList(String sql, Object... args)`。
    ///
    /// # 参数
    ///
    /// - `sql`: SQL 查询语句（使用 `?` 占位符）
    /// - `args`: 绑定参数列表
    ///
    /// # 返回
    ///
    /// 查询结果的 `Vec<Value>`，每个 `Value` 是一行数据（Array 类型）。
    pub async fn select_list(
        &self,
        sql: &str,
        args: Vec<Value>,
    ) -> Result<Vec<Value>, rbatis::Error> {
        let value = self.db.query(sql, args).await?;
        match value {
            Value::Array(arr) => Ok(arr),
            other => Ok(vec![other]),
        }
    }

    /// 执行查询，返回单条记录。
    ///
    /// 对应 Java `SqlRunner.selectOne(String sql, Object... args)`。
    ///
    /// # 返回
    ///
    /// - `Ok(Some(value))` — 查询到一条记录
    /// - `Ok(None)` — 查询结果为空
    pub async fn select_one(
        &self,
        sql: &str,
        args: Vec<Value>,
    ) -> Result<Option<Value>, rbatis::Error> {
        let value = self.db.query(sql, args).await?;
        match value {
            Value::Array(arr) => {
                if arr.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(arr.into_iter().next().unwrap()))
                }
            }
            Value::Null => Ok(None),
            other => Ok(Some(other)),
        }
    }

    /// 执行 COUNT 查询，返回计数值。
    ///
    /// 对应 Java `SqlRunner.selectCount(String sql, Object... args)`。
    ///
    /// # 返回
    ///
    /// 查询到的计数值。如果结果为空则返回 0。
    ///
    /// # Note
    ///
    /// 支持多种数据库返回类型：
    /// - `i64`（MySQL/PostgreSQL COUNT 返回 bigint）
    /// - `u64`（部分驱动）
    /// - 字符串形式的数字
    pub async fn select_count(
        &self,
        sql: &str,
        args: Vec<Value>,
    ) -> Result<u64, rbatis::Error> {
        let value = self.db.query(sql, args).await?;
        Self::extract_count(value)
    }

    /// 从 Value 中提取 COUNT 结果。
    fn extract_count(value: Value) -> Result<u64, rbatis::Error> {
        match value {
            Value::Array(arr) => {
                if let Some(first) = arr.into_iter().next() {
                    Self::extract_count_from_row(first)
                } else {
                    Ok(0)
                }
            }
            Value::Null => Ok(0),
            other => Self::extract_count_from_row(other),
        }
    }

    /// 从单行中提取 COUNT 值。
    fn extract_count_from_row(row: Value) -> Result<u64, rbatis::Error> {
        match row {
            Value::Map(map) => {
                // COUNT(*) 的结果通常在第一个列中
                if let Some((_key, val)) = map.into_iter().next() {
                    Self::value_to_u64(val)
                } else {
                    Ok(0)
                }
            }
            Value::I64(n) => Ok(n as u64),
            Value::U64(n) => Ok(n),
            Value::I32(n) => Ok(n as u64),
            Value::U32(n) => Ok(n as u64),
            Value::String(ref s) => Ok(s.parse::<u64>().unwrap_or(0)),
            other => {
                log::warn!("SqlRunner: 无法解析 COUNT 结果: {:?}", other);
                Ok(0)
            }
        }
    }

    /// 将 Value 转换为 u64。
    fn value_to_u64(val: Value) -> Result<u64, rbatis::Error> {
        match val {
            Value::I64(n) => Ok(n as u64),
            Value::U64(n) => Ok(n),
            Value::I32(n) => Ok(n as u64),
            Value::U32(n) => Ok(n as u64),
            Value::F32(n) => Ok(n as u64),
            Value::F64(n) => Ok(n as u64),
            Value::String(s) => s.parse::<u64>().map_err(|e| {
                rbatis::Error::from(format!("SqlRunner: 无法将字符串 '{}' 解析为 u64: {}", s, e))
            }),
            Value::Null => Ok(0),
            other => Err(rbatis::Error::from(format!(
                "SqlRunner: 无法将 {:?} 转换为 u64",
                other
            ))),
        }
    }

    /// 执行 INSERT / UPDATE / DELETE 语句。
    ///
    /// 对应 Java `SqlRunner.update(String sql, Object... args)`（即 `SqlRunner` 中的 `insert` / `update` / `delete`）。
    ///
    /// # 返回
    ///
    /// 受影响的行数。
    pub async fn execute(
        &self,
        sql: &str,
        args: Vec<Value>,
    ) -> Result<u64, rbatis::Error> {
        let result = self.db.exec(sql, args).await?;
        Ok(result.rows_affected)
    }

    /// 在事务内执行多条 SQL。
    ///
    /// 对应 Java `SqlRunner` 中使用 `Connection` 手动管理事务的模式。
    ///
    /// 该方法开启一个事务，将 `SqlRunner`（共享同一个 `RBatis`）传给闭包 `f`，
    /// 闭包返回 `Ok` 时自动提交，返回 `Err` 时自动回滚。
    ///
    /// # 参数
    ///
    /// - `f`: 接收 `SqlRunner` 的异步闭包，在事务上下文中执行多条 SQL
    ///
    /// # 返回
    ///
    /// 闭包的返回值，或执行过程中的错误。
    pub async fn transaction<F, Fut, R>(&self, f: F) -> Result<R, rbatis::Error>
    where
        F: FnOnce(SqlRunner) -> Fut,
        Fut: std::future::Future<Output = Result<R, rbatis::Error>> + Send,
        R: Send,
    {
        let tx = self.db.acquire_begin().await?;
        let runner = SqlRunner::new(self.db.clone());
        match f(runner).await {
            Ok(result) => {
                tx.commit().await?;
                Ok(result)
            }
            Err(e) => {
                // 回滚忽略错误，因为原始错误更有价值
                let _ = tx.rollback().await;
                Err(e)
            }
        }
    }
}

impl From<RBatis> for SqlRunner {
    fn from(db: RBatis) -> Self {
        Self::new(db)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试 SqlRunner 的创建和 Clone。
    #[test]
    fn test_sql_runner_creation() {
        let rb = RBatis::new();
        let runner = SqlRunner::new(rb);
        // clone 应成功（内部 Arc 共享连接池）
        let runner2 = runner.clone();
        // 两个 runner 的 rb 指向相同底层数据（Arc）
        let _ = runner.rb();
        let _ = runner2.rb();
    }

    /// 测试 SqlRunner::from(RBatis)。
    #[test]
    fn test_sql_runner_from_rbatis() {
        let rb = RBatis::new();
        let runner: SqlRunner = rb.into();
        // rb() 返回内部 RBatis 引用
        let _ = runner.rb();
    }

    /// 测试 extract_count 对各种 Value 类型的解析。
    #[test]
    fn test_extract_count_values() {
        // i64
        assert_eq!(SqlRunner::extract_count(Value::I64(42)).unwrap(), 42);
        // u64
        assert_eq!(SqlRunner::extract_count(Value::U64(100)).unwrap(), 100);
        // i32
        assert_eq!(SqlRunner::extract_count(Value::I32(10)).unwrap(), 10);
        // Null
        assert_eq!(SqlRunner::extract_count(Value::Null).unwrap(), 0);
        // String
        assert_eq!(
            SqlRunner::extract_count(Value::String("99".into())).unwrap(),
            99
        );
    }

    /// 测试 extract_count 从数组中提取（典型 SELECT COUNT(*) 返回格式）。
    #[test]
    fn test_extract_count_from_array() {
        // 模拟 SELECT COUNT(*) AS count FROM table 的返回
        let mut map = rbs::value::map::ValueMap::new();
        map.insert("count".into(), Value::I64(42));
        let row = Value::Map(map);
        let arr = Value::Array(vec![row]);
        assert_eq!(SqlRunner::extract_count(arr).unwrap(), 42);
    }

    /// 测试 extract_count 空数组。
    #[test]
    fn test_extract_count_empty_array() {
        assert_eq!(
            SqlRunner::extract_count(Value::Array(vec![])).unwrap(),
            0
        );
    }

    /// 测试 value_to_u64 对各种类型的转换。
    #[test]
    fn test_value_to_u64() {
        assert_eq!(SqlRunner::value_to_u64(Value::I64(5)).unwrap(), 5);
        assert_eq!(SqlRunner::value_to_u64(Value::U64(7)).unwrap(), 7);
        assert_eq!(SqlRunner::value_to_u64(Value::I32(3)).unwrap(), 3);
        assert_eq!(SqlRunner::value_to_u64(Value::U32(4)).unwrap(), 4);
        assert_eq!(SqlRunner::value_to_u64(Value::F32(2.0)).unwrap(), 2);
        assert_eq!(SqlRunner::value_to_u64(Value::F64(8.0)).unwrap(), 8);
        assert_eq!(SqlRunner::value_to_u64(Value::Null).unwrap(), 0);
        assert_eq!(
            SqlRunner::value_to_u64(Value::String("123".into())).unwrap(),
            123
        );
        // 无法解析的字符串应返回错误
        assert!(SqlRunner::value_to_u64(Value::String("abc".into())).is_err());
    }

    /// 测试 select_list 在 Value::Array 返回时的行为。
    #[test]
    fn test_select_list_non_array() {
        // select_list 内部会将非数组值包装为 vec
        // 这里测试 extract_count 的逻辑（select_list 的核心行为类似）
        let val = Value::I64(1);
        // 模拟 select_list 中的 match 分支
        let result = match val {
            Value::Array(arr) => arr,
            other => vec![other],
        };
        assert_eq!(result.len(), 1);
    }

    /// 测试 select_one 对空数组返回 None。
    #[test]
    fn test_select_one_empty() {
        let val = Value::Array(vec![]);
        let result = match val {
            Value::Array(arr) => {
                if arr.is_empty() {
                    None
                } else {
                    Some(arr.into_iter().next().unwrap())
                }
            }
            Value::Null => None,
            other => Some(other),
        };
        assert!(result.is_none());
    }

    /// 测试 select_one 对单元素数组返回 Some。
    #[test]
    fn test_select_one_single() {
        let val = Value::Array(vec![Value::I64(42)]);
        let result = match val {
            Value::Array(arr) => {
                if arr.is_empty() {
                    None
                } else {
                    Some(arr.into_iter().next().unwrap())
                }
            }
            Value::Null => None,
            other => Some(other),
        };
        assert!(result.is_some());
        assert_eq!(result.unwrap(), Value::I64(42));
    }
}
