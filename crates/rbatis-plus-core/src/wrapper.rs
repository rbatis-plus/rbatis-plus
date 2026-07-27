// Source: rbatis-wrapper/src/wrapper.rs (absorbed for backward compat)

use crate::conditions::query::QueryWrapper;
use crate::conditions::join::Join;
use crate::page::{Page, PageRequest};
use rbatis::executor::Executor;
use rbatis::Error;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Legacy compatibility entry point — absorbs the `rbatis-wrapper` crate API.
///
/// In new code prefer `QueryWrapper` directly; this module provides
/// the same `query()`, `get_one()`, `delete()`, and `page()` async methods
/// that `rbatis-wrapper` exposed, now backed by the full `QueryWrapper`.
impl QueryWrapper {
    /// Execute the query and return a list of entities.
    ///
    /// 执行查询返回列表
    pub async fn query<T: Serialize + DeserializeOwned>(
        &self,
        rb: &dyn Executor,
        table_name: &str,
    ) -> Result<Vec<T>, Error> {
        let sql = self.build_select_sql(table_name);
        let v = rb.query(&sql, vec![]).await?;
        Ok(rbs::from_value(v)?)
    }

    /// Execute the query and return a single entity (or None).
    ///
    /// 执行查询返回单条
    pub async fn get_one<T: Serialize + DeserializeOwned>(
        &self,
        rb: &dyn Executor,
        table_name: &str,
    ) -> Result<Option<T>, Error> {
        // Add LIMIT 1 if not already set
        let mut clone = self.clone();
        if clone.inner.sql_last.is_empty() {
            clone.last("LIMIT 1");
        }
        let sql = clone.build_select_sql(table_name);
        let v = rb.query(&sql, vec![]).await?;
        let list: Vec<T> = rbs::from_value(v)?;
        Ok(list.into_iter().next())
    }

    /// Execute a paginated query.
    ///
    /// 执行分页查询
    pub async fn page<T: Serialize + DeserializeOwned>(
        &self,
        rb: &dyn Executor,
        table_name: &str,
        page_no: u64,
        page_size: u64,
    ) -> Result<Page<T>, Error> {
        let req = PageRequest::new(page_no, page_size);

        // Count
        let count_sql = self.build_count_sql(table_name);
        let v = rb.query(&count_sql, vec![]).await?;
        let total: u64 = rbs::from_value(v).unwrap_or(0);

        if total == 0 {
            return Ok(Page::empty(page_no, page_size));
        }

        // Data
        let mut clone = self.clone();
        clone.last(&format!("LIMIT {} OFFSET {}", req.page_size, req.offset()));
        let sql = clone.build_select_sql(table_name);
        let v = rb.query(&sql, vec![]).await?;
        let records: Vec<T> = rbs::from_value(v)?;

        Ok(Page::new(records, total, page_no, page_size))
    }
}
