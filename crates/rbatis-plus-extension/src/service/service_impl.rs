// Source: mybatis-plus-extension/.../service/ServiceImpl.java

use super::i_service::IService;
use async_trait::async_trait;
use rbatis::executor::Executor;
use rbatis::Error;
use rbatis_plus_core::conditions::query::QueryWrapper;
use rbatis_plus_core::conditions::update::UpdateWrapper;
use rbatis_plus_core::page::Page;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::PhantomData;

/// Service 实现基类（对标 Java `ServiceImpl<M, T>`）。
///
/// 提供 `IService<T>` trait 的默认实现，基于 rbatis `Executor` 执行数据库操作。
///
/// - `E` — Executor 类型（`RBatis`、`RBatisConnExecutor`、`RBatisTxExecutor` 等）
/// - `T` — 实体类型
///
/// # 对应 Java
///
/// - `com.baomidou.mybatisplus.extension.service.impl.ServiceImpl<M, T>`
///
/// # Example
///
/// ```ignore
/// use rbatis_plus_extension::service::{ServiceImpl, IService};
///
/// struct UserService {
///     inner: ServiceImpl<rbatis::RBatis, User>,
/// }
///
/// impl UserService {
///     pub fn new(rb: rbatis::RBatis) -> Self {
///         Self {
///             inner: ServiceImpl::new(rb, "sys_user"),
///         }
///     }
/// }
/// ```
pub struct ServiceImpl<E, T>
where
    E: Executor + Send + Sync + Clone + 'static,
    T: Serialize + DeserializeOwned + Send + Sync,
{
    /// 数据库执行器。
    executor: E,
    /// 表名。
    table_name: String,
    /// 主键列名（默认 "id"）。
    id_column: String,
    /// 实体类型幽灵标记。
    _phantom: PhantomData<T>,
}

impl<E, T> ServiceImpl<E, T>
where
    E: Executor + Send + Sync + Clone + 'static,
    T: Serialize + DeserializeOwned + Send + Sync,
{
    /// 创建 ServiceImpl 实例。
    ///
    /// # 参数
    /// - `executor`: 数据库执行器（RBatis / ConnExecutor / TxExecutor）
    /// - `table_name`: 数据库表名
    pub fn new(executor: E, table_name: impl Into<String>) -> Self {
        Self {
            executor,
            table_name: table_name.into(),
            id_column: "id".to_string(),
            _phantom: PhantomData,
        }
    }

    /// 设置主键列名（默认 "id"）。
    pub fn with_id_column(mut self, column: impl Into<String>) -> Self {
        self.id_column = column.into();
        self
    }

    /// 获取表名。
    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    /// 获取主键列名。
    pub fn id_column(&self) -> &str {
        &self.id_column
    }

    /// 获取执行器引用。
    pub fn executor(&self) -> &E {
        &self.executor
    }

    /// 内部：从 Value Map 中提取主键值。
    fn extract_id(map: &rbs::Value, id_column: &str) -> rbs::Value {
        if let rbs::Value::Map(m) = map {
            for (k, v) in m {
                if let rbs::Value::String(col_name) = k {
                    if col_name == id_column {
                        return v.clone();
                    }
                }
            }
        }
        rbs::Value::Null
    }

    /// 内部：将 rbs::Value 解码为 Vec<T>。
    fn decode_rows(value: rbs::Value) -> Result<Vec<T>, Error> {
        match value {
            rbs::Value::Array(arr) => {
                let mut result = Vec::with_capacity(arr.len());
                for item in arr {
                    let row: T = rbs::from_value(item)?;
                    result.push(row);
                }
                Ok(result)
            }
            _ => Ok(Vec::new()),
        }
    }

    /// 内部：从 Value Map 中提取 u64 总数。
    fn extract_count(value: &rbs::Value) -> u64 {
        match value {
            rbs::Value::Map(map) => {
                for (_, v) in map {
                    match v {
                        rbs::Value::U64(n) => return *n,
                        rbs::Value::I64(n) => return *n as u64,
                        rbs::Value::U32(n) => return *n as u64,
                        rbs::Value::I32(n) => return *n as u64,
                        _ => {}
                    }
                }
                0
            }
            rbs::Value::U64(n) => *n,
            rbs::Value::I64(n) => *n as u64,
            _ => 0,
        }
    }
}

#[async_trait]
impl<E, T> IService<T> for ServiceImpl<E, T>
where
    E: Executor + Send + Sync + Clone + 'static,
    T: Serialize + DeserializeOwned + Send + Sync,
{
    /// 插入一条记录（对标 Java `ServiceImpl.save()`）。
    async fn save(&self, entity: &T) -> Result<bool, Error> {
        let map = rbs::to_value(entity)?;
        let (cols, placeholders, values) = Self::build_insert_params(&map, &self.id_column);
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.table_name, cols, placeholders
        );
        let result = self.executor.exec(&sql, values).await?;
        Ok(result.rows_affected > 0)
    }

    /// 插入或更新（根据 ID 判断）（对标 Java `ServiceImpl.saveOrUpdate()`）。
    async fn save_or_update(&self, entity: &T) -> Result<bool, Error> {
        let map = rbs::to_value(entity)?;
        let id = Self::extract_id(&map, &self.id_column);
        if !matches!(id, rbs::Value::Null) {
            let (set_clause, values) = Self::build_update_params(&map, &self.id_column);
            let sql = format!(
                "UPDATE {} SET {} WHERE {} = ?",
                self.table_name, set_clause, self.id_column
            );
            let mut all_values = values;
            all_values.push(id);
            let result = self.executor.exec(&sql, all_values).await?;
            if result.rows_affected > 0 {
                return Ok(true);
            }
        }
        self.save(entity).await
    }

    /// 根据 ID 删除（对标 Java `ServiceImpl.removeById()`）。
    async fn remove_by_id(&self, id: &rbs::Value) -> Result<bool, Error> {
        let sql = format!("DELETE FROM {} WHERE {} = ?", self.table_name, self.id_column);
        let result = self.executor.exec(&sql, vec![id.clone()]).await?;
        Ok(result.rows_affected > 0)
    }

    /// 根据条件删除（对标 Java `ServiceImpl.remove(Wrapper)`）。
    async fn remove(&self, wrapper: &QueryWrapper, table_name: &str) -> Result<bool, Error> {
        let where_clause = wrapper.inner.build_where();
        let sql = format!("DELETE FROM {}{}", table_name, where_clause);
        let result = self.executor.exec(&sql, wrapper.params().to_vec()).await?;
        Ok(result.rows_affected > 0)
    }

    /// 根据 ID 更新（对标 Java `ServiceImpl.updateById()`）。
    async fn update_by_id(&self, entity: &T) -> Result<bool, Error> {
        let map = rbs::to_value(entity)?;
        let id = Self::extract_id(&map, &self.id_column);
        let (set_clause, values) = Self::build_update_params(&map, &self.id_column);
        let sql = format!(
            "UPDATE {} SET {} WHERE {} = ?",
            self.table_name, set_clause, self.id_column
        );
        let mut all_values = values;
        all_values.push(id);
        let result = self.executor.exec(&sql, all_values).await?;
        Ok(result.rows_affected > 0)
    }

    /// 根据条件更新（对标 Java `ServiceImpl.update(Wrapper)`）。
    async fn update(
        &self,
        wrapper: &UpdateWrapper,
        table_name: &str,
    ) -> Result<bool, Error> {
        let sql = wrapper.build_update_sql(table_name);
        let result = self.executor.exec(&sql, wrapper.params().to_vec()).await?;
        Ok(result.rows_affected > 0)
    }

    /// 根据 ID 查询（对标 Java `ServiceImpl.getById()`）。
    async fn get_by_id(&self, id: &rbs::Value) -> Result<Option<T>, Error> {
        let sql = format!(
            "SELECT * FROM {} WHERE {} = ? LIMIT 1",
            self.table_name, self.id_column
        );
        let value = self.executor.query(&sql, vec![id.clone()]).await?;
        let rows = Self::decode_rows(value)?;
        Ok(rows.into_iter().next())
    }

    /// 根据条件查询列表（对标 Java `ServiceImpl.list(Wrapper)`）。
    async fn list(&self, wrapper: &QueryWrapper, table_name: &str) -> Result<Vec<T>, Error> {
        let sql = wrapper.build_select_sql(table_name);
        let value = self.executor.query(&sql, wrapper.params().to_vec()).await?;
        Self::decode_rows(value)
    }

    /// 根据条件查询单条（对标 Java `ServiceImpl.getOne(Wrapper)`）。
    async fn get_one(
        &self,
        wrapper: &QueryWrapper,
        table_name: &str,
    ) -> Result<Option<T>, Error> {
        let sql = format!("{} LIMIT 1", wrapper.build_select_sql(table_name));
        let value = self.executor.query(&sql, wrapper.params().to_vec()).await?;
        let rows = Self::decode_rows(value)?;
        Ok(rows.into_iter().next())
    }

    /// 查询总数（对标 Java `ServiceImpl.count(Wrapper)`）。
    async fn count(&self, wrapper: &QueryWrapper, table_name: &str) -> Result<u64, Error> {
        let sql = wrapper.build_count_sql(table_name);
        let value = self.executor.query(&sql, wrapper.params().to_vec()).await?;
        Ok(Self::extract_count(&value))
    }

    /// 分页查询（对标 Java `ServiceImpl.page(Wrapper, Page)`）。
    async fn page(
        &self,
        wrapper: &QueryWrapper,
        table_name: &str,
        page_no: u64,
        page_size: u64,
    ) -> Result<Page<T>, Error> {
        // 先查询总数
        let total = self.count(wrapper, table_name).await?;

        // 再查询分页数据
        let offset = (page_no - 1) * page_size;
        let sql = format!(
            "{} LIMIT {}, {}",
            wrapper.build_select_sql(table_name),
            offset,
            page_size
        );
        let value = self.executor.query(&sql, wrapper.params().to_vec()).await?;
        let records = Self::decode_rows(value)?;

        Ok(Page::new(records, total, page_no, page_size))
    }
}

impl<E, T> ServiceImpl<E, T>
where
    E: Executor + Send + Sync + Clone + 'static,
    T: Serialize + DeserializeOwned + Send + Sync,
{
    /// 构建 INSERT 参数（列名、占位符、值列表）。
    fn build_insert_params(map: &rbs::Value, id_column: &str) -> (String, String, Vec<rbs::Value>) {
        let mut cols = Vec::new();
        let mut placeholders = Vec::new();
        let mut values = Vec::new();

        if let rbs::Value::Map(m) = map {
            for (k, v) in m {
                if let rbs::Value::String(col_name) = k {
                    // 跳过 null 的自增主键
                    if col_name == id_column && matches!(v, rbs::Value::Null) {
                        continue;
                    }
                    cols.push(col_name.clone());
                    placeholders.push("?".to_string());
                    values.push(v.clone());
                }
            }
        }

        (cols.join(", "), placeholders.join(", "), values)
    }

    /// 构建 UPDATE SET 参数（set 子句、值列表，不含主键）。
    fn build_update_params(map: &rbs::Value, id_column: &str) -> (String, Vec<rbs::Value>) {
        let mut set_parts = Vec::new();
        let mut values = Vec::new();

        if let rbs::Value::Map(m) = map {
            for (k, v) in m {
                if let rbs::Value::String(col_name) = k {
                    if col_name == id_column {
                        continue; // 主键放在 WHERE 中
                    }
                    set_parts.push(format!("{} = ?", col_name));
                    values.push(v.clone());
                }
            }
        }

        (set_parts.join(", "), values)
    }
}
