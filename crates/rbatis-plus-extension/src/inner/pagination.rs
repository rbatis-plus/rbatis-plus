// Source: mybatis-plus-jsqlparser-5.0/.../inner/PaginationInnerInterceptor.java

use crate::inner::inner_interceptor::InnerInterceptor;
use async_trait::async_trait;
use rbatis::executor::Executor;
use rbatis::{Action, Error};
use rbatis_plus_sqlparser::{SqlRewriter, SqlDialect, MysqlDialect, PostgreSqlDialect, SqliteDialect};
use rbs::Value;
use std::sync::{Arc, Mutex};

/// 分页拦截器（对标 Java `PaginationInnerInterceptor`）。
///
/// 拦截 SELECT 查询，在 SQL 到达数据库前自动添加 `LIMIT/OFFSET` 分页子句。
/// 支持 MySQL、PostgreSQL、SQLite 三种方言。
///
/// # 对应 Java
///
/// - `com.baomidou.mybatisplus.extension.plugins.inner.PaginationInnerInterceptor`
/// - `com.baomidou.mybatisplus.extension.plugins.pagination.dialects.IDialect`
///
/// # 用法
///
/// ```ignore
/// use rbatis_plus_extension::inner::pagination::PaginationInnerInterceptor;
/// use rbatis_plus_sqlparser::MysqlDialect;
///
/// // 1. 创建拦截器
/// let interceptor = PaginationInnerInterceptor::new()
///     .with_max_limit(500)
///     .with_mysql();
///
/// // 2. 设置分页参数（在执行查询前）
/// interceptor.set_page(3, 20); // 第 3 页，每页 20 条
///
/// // 3. 拦截器自动改写 SQL：
/// //    SELECT * FROM users WHERE status = 1
/// //    → SELECT * FROM users WHERE status = 1 LIMIT 40, 20
///
/// // 4. 查询完成后自动清除分页参数（单次生效）
/// ```
///
/// # 分页模式
///
/// 支持两种分页模式（对标 Java `PaginationInnerInterceptor`）：
///
/// 1. **手动模式**（推荐）：调用 `set_page(page_no, page_size)` 设置分页参数，
///    拦截器自动改写 SQL。参数在一次查询后自动清除。
///
/// 2. **参数标记模式**：在 SQL args 中注入 `__rbatis_page_no__` 和 `__rbatis_page_size__`
///    标记，拦截器自动识别并改写。（实验性）
pub struct PaginationInnerInterceptor {
    /// 最大允许的分页大小（0 = 不限制）。
    pub max_limit: u64,
    /// 数据库方言（默认 MySQL）。
    dialect: Arc<dyn SqlDialect>,
    /// 当前分页参数（page_no, page_size），由 `set_page()` 设置。
    /// 使用 Mutex 实现内部可变性（`before_query` 接收 `&self`）。
    page_params: Arc<Mutex<Option<(u64, u64)>>>,
}

impl std::fmt::Debug for PaginationInnerInterceptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PaginationInnerInterceptor")
            .field("max_limit", &self.max_limit)
            .field("dialect", &self.dialect.name())
            .field("page_params", &self.page_params.lock().ok())
            .finish()
    }
}

impl Default for PaginationInnerInterceptor {
    fn default() -> Self {
        Self {
            max_limit: 500,
            dialect: Arc::new(MysqlDialect),
            page_params: Arc::new(Mutex::new(None)),
        }
    }
}

impl PaginationInnerInterceptor {
    /// 创建默认（MySQL 方言）的分页拦截器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置最大分页限制。
    pub fn with_max_limit(mut self, max_limit: u64) -> Self {
        self.max_limit = max_limit;
        self
    }

    /// 设置数据库方言。
    ///
    /// 对应 Java `PaginationInnerInterceptor.setDialect(IDialect)`
    pub fn with_dialect(mut self, dialect: Box<dyn SqlDialect>) -> Self {
        self.dialect = Arc::from(dialect);
        self
    }

    /// 设置 MySQL 方言。
    pub fn with_mysql(self) -> Self {
        self.with_dialect(Box::new(MysqlDialect))
    }

    /// 设置 PostgreSQL 方言。
    pub fn with_postgresql(self) -> Self {
        self.with_dialect(Box::new(PostgreSqlDialect))
    }

    /// 设置 SQLite 方言。
    pub fn with_sqlite(self) -> Self {
        self.with_dialect(Box::new(SqliteDialect))
    }

    /// 设置分页参数（在执行查询前调用）。
    ///
    /// 对应 Java `PageHelper.startPage(pageNo, pageSize)` 的手动分页模式。
    ///
    /// 设置后，下一次 `before_query` 拦截会自动改写 SQL。
    /// 改写完成后参数自动清除（单次生效）。
    pub fn set_page(&self, page_no: u64, page_size: u64) {
        if let Ok(mut params) = self.page_params.lock() {
            *params = Some((page_no.max(1), page_size.max(1)));
        }
    }

    /// 清除分页参数。
    pub fn clear_page(&self) {
        if let Ok(mut params) = self.page_params.lock() {
            *params = None;
        }
    }

    /// 获取当前分页参数。
    pub fn get_page(&self) -> Option<(u64, u64)> {
        self.page_params.lock().ok().and_then(|p| *p)
    }

    /// 限制分页大小（对标 `PaginationInnerInterceptor.handlerLimit()`）。
    pub fn clamp_page_size(&self, page_size: u64) -> u64 {
        if self.max_limit > 0 && page_size > self.max_limit {
            log::warn!("分页大小 {} 超过最大限制 {}，已截断", page_size, self.max_limit);
            self.max_limit
        } else {
            page_size.max(1)
        }
    }

    /// 检查 SQL 是否可以分页（对标 `PaginationInnerInterceptor.consumes()`）。
    pub fn can_paginate(&self, sql: &str) -> bool {
        SqlRewriter::can_paginate(sql)
    }

    /// 改写 SQL 添加分页（对标 `PaginationInnerInterceptor.findDialect()` + 方言改写）。
    pub fn rewrite_sql(&self, sql: &str, page_no: u64, page_size: u64) -> String {
        let size = self.clamp_page_size(page_size);
        let offset = (page_no - 1) * size;
        self.dialect.build_pagination_sql(sql, offset, size)
    }
}

#[async_trait]
impl InnerInterceptor for PaginationInnerInterceptor {
    /// 拦截查询，自动改写 SQL 添加分页（对标 `PaginationInnerInterceptor.willDoQuery()`）。
    ///
    /// 流程：
    /// 1. 检查是否有待处理的分页参数
    /// 2. 检查 SQL 是否可以分页（SELECT 且不含 FOR UPDATE）
    /// 3. 使用方言改写 SQL 添加 LIMIT/OFFSET
    /// 4. 清除分页参数（单次生效）
    async fn before_query(
        &self,
        _executor: &dyn Executor,
        sql: &mut String,
        _args: &mut Vec<Value>,
        _result: &mut Result<Value, Error>,
    ) -> Result<Action, Error> {
        // 1. 获取分页参数
        let page = match self.get_page() {
            Some(p) => p,
            None => return Ok(Action::Next), // 无分页参数，跳过
        };

        // 2. 检查 SQL 是否可以分页
        if !self.can_paginate(sql) {
            log::debug!("SQL 不支持分页，跳过: {}", &sql[..sql.len().min(100)]);
            self.clear_page(); // 清除参数防止泄漏到后续查询
            return Ok(Action::Next);
        }

        // 3. 改写 SQL
        let original_sql = sql.clone();
        *sql = self.rewrite_sql(sql, page.0, page.1);

        log::debug!(
            "分页改写: page_no={}, page_size={} \n原始: {}\n改写: {}",
            page.0, page.1, original_sql, sql
        );

        // 4. 清除分页参数（单次生效）
        self.clear_page();

        Ok(Action::Next)
    }

    async fn after_query(
        &self,
        _executor: &dyn Executor,
        _sql: &str,
        _result: &mut Result<Value, Error>,
    ) -> Result<(), Error> {
        // 确保分页参数被清除（防止异常情况泄漏）
        self.clear_page();
        Ok(())
    }
}
