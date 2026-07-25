//! axum 框架集成（对标 mybatis-plus-spring 的 Spring MVC 集成）。
//!
//! 提供 axum 的分页提取器、事务中间件等集成能力。
//!
//! # 对应 Java
//!
//! - `com.baomidou.mybatisplus.extension.plugins.pagination.Page`
//! - `com.baomidou.mybatisplus.extension.plugins.inner.PaginationInnerInterceptor`

use rbatis_plus_core::page::{Page, PageRequest};
use serde::Deserialize;

/// axum 分页查询参数提取器（对标 Java `Page<T>` 的 HTTP 参数绑定）。
///
/// 从 URL query string 中自动提取分页参数：
/// - `page_no`（默认 1）
/// - `page_size`（默认 10）
///
/// # Example
///
/// ```ignore
/// use axum::extract::Query;
/// use rbatis_plus_vernal::axum_integration::PageParam;
///
/// async fn list(Query(param): Query<PageParam>) -> String {
///     let page_req = param.to_page_request(500); // max_page_size = 500
///     format!("page={}, size={}", page_req.page_no, page_req.page_size)
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct PageParam {
    /// 页码（从 1 开始）。
    #[serde(default = "default_page_no")]
    pub page_no: u64,
    /// 每页大小。
    #[serde(default = "default_page_size")]
    pub page_size: u64,
}

fn default_page_no() -> u64 { 1 }
fn default_page_size() -> u64 { 10 }

impl PageParam {
    /// 转换为 PageRequest（带最大页大小限制）。
    ///
    /// 对应 Java `PaginationInnerInterceptor.autoPage()` 中的大小限制逻辑。
    pub fn to_page_request(&self, max_page_size: u64) -> PageRequest {
        let size = if self.page_size > max_page_size {
            log::warn!("分页大小 {} 超过最大限制 {}，已截断", self.page_size, max_page_size);
            max_page_size
        } else if self.page_size == 0 {
            10
        } else {
            self.page_size
        };
        let page_no = if self.page_no == 0 { 1 } else { self.page_no };
        PageRequest::new(page_no, size)
    }

    /// 创建空的分页结果。
    pub fn empty_page<T>(&self) -> Page<T> {
        Page::empty(self.page_no, self.page_size)
    }
}

impl Default for PageParam {
    fn default() -> Self {
        Self {
            page_no: 1,
            page_size: 10,
        }
    }
}

/// 排序参数提取器（对标 Java `ISqlParser` 中的 ORDER BY 处理）。
///
/// 从 URL query string 中提取排序参数：
/// - `order_by`（列名）
/// - `order`（`asc` 或 `desc`，默认 `asc`）
#[derive(Debug, Clone, Deserialize)]
pub struct OrderParam {
    /// 排序列名。
    #[serde(default)]
    pub order_by: Option<String>,
    /// 排序方向（`asc` / `desc`）。
    #[serde(default)]
    pub order: Option<String>,
}

impl OrderParam {
    /// 是否有排序。
    pub fn has_order(&self) -> bool {
        self.order_by.is_some()
    }

    /// 获取排序方向（默认 ASC）。
    pub fn is_asc(&self) -> bool {
        self.order.as_deref() != Some("desc")
    }

    /// 构建 ORDER BY 子句。
    pub fn build_order_by(&self) -> String {
        match &self.order_by {
            Some(col) => {
                let dir = if self.is_asc() { "ASC" } else { "DESC" };
                format!(" ORDER BY {} {}", col, dir)
            }
            None => String::new(),
        }
    }
}
