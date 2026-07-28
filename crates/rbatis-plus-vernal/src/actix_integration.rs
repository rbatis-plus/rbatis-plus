//! actix-web 框架集成（对标 mybatis-plus-spring 的 Spring MVC 集成）。
//!
//! 提供 actix-web 的分页提取器、分页响应体等集成能力。
//!
//! # 对应 Java
//!
//! - `com.baomidou.mybatisplus.extension.plugins.pagination.Page`
//! - `com.baomidou.mybatisplus.extension.plugins.inner.PaginationInnerInterceptor`

use actix_web::HttpRequest;
use actix_web::Responder;
use rbatis_plus_core::page::Page;
use serde::Deserialize;

/// actix-web 分页查询参数提取器（对标 Java `Page<T>` 的 HTTP 参数绑定）。
///
/// 从 URL query string 中自动提取分页参数：
/// - `page_no`（默认 1）
/// - `page_size`（默认 10）
///
/// # Example
///
/// ```ignore
/// use actix_web::web;
/// use rbatis_plus_vernal::actix_integration::PageParam;
///
/// async fn list(query: web::Query<PageParam>) -> String {
///     let param = query.into_inner();
///     format!("page={}, size={}", param.page_no, param.page_size)
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

fn default_page_no() -> u64 {
    1
}
fn default_page_size() -> u64 {
    10
}

impl PageParam {
    /// 转换为 `PageRequest`（带最大页大小限制）。
    ///
    /// 对应 Java `PaginationInnerInterceptor.autoPage()` 中的大小限制逻辑。
    pub fn to_page_request(
        &self,
        max_page_size: u64,
    ) -> rbatis_plus_core::page::PageRequest {
        let size = if self.page_size > max_page_size {
            log::warn!(
                "分页大小 {} 超过最大限制 {}，已截断",
                self.page_size,
                max_page_size
            );
            max_page_size
        } else if self.page_size == 0 {
            10
        } else {
            self.page_size
        };
        let page_no = if self.page_no == 0 { 1 } else { self.page_no };
        rbatis_plus_core::page::PageRequest::new(page_no, size)
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

/// actix-web `FromRequest` 实现，从 query string 中提取 `PageParam`。
///
/// 对应 Java Spring MVC 中 `@RequestParam` 自动绑定分页参数。
impl actix_web::FromRequest for PageParam {
    type Error = actix_web::Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let query = req.query_string();
        let param: PageParam = serde_urlencoded::from_str(query).unwrap_or_default();
        std::future::ready(Ok(param))
    }
}

/// 分页响应包装器（对标 Java `@ResponseBody` + `Page<T>`）。
///
/// 由于 Rust orphan rule 限制，无法直接为 `Page<T>` 实现 actix-web 的 `Responder`。
/// 使用此包装器将 `Page<T>` 转换为 JSON 响应。
///
/// # Example
///
/// ```ignore
/// use rbatis_plus_vernal::actix_integration::PageResponse;
/// use rbatis_plus_core::page::Page;
///
/// async fn list() -> PageResponse<MyItem> {
///     let page: Page<MyItem> = query_page().await;
///     PageResponse(page)
/// }
/// ```
pub struct PageResponse<T: serde::Serialize>(pub Page<T>);

impl<T: serde::Serialize> Responder for PageResponse<T> {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        match serde_json::to_value(&self.0) {
            Ok(json) => actix_web::HttpResponse::Ok()
                .content_type("application/json")
                .body(json.to_string()),
            Err(e) => {
                log::error!("分页结果序列化失败: {}", e);
                actix_web::HttpResponse::InternalServerError()
                    .body(format!("分页结果序列化失败: {}", e))
            }
        }
    }
}

/// 辅助函数：将 `Page<T>` 转换为 actix-web JSON 响应。
///
/// 对应 Java Spring MVC 中的 `ResponseEntity.ok(page)`。
///
/// # Example
///
/// ```ignore
/// use rbatis_plus_vernal::actix_integration::{page_response, PageParam};
/// use rbatis_plus_core::page::Page;
///
/// async fn list() -> actix_web::HttpResponse {
///     let page: Page<MyItem> = query_page().await;
///     page_response(page)
/// }
/// ```
pub fn page_response<T: serde::Serialize>(page: Page<T>) -> actix_web::HttpResponse {
    match serde_json::to_value(&page) {
        Ok(json) => actix_web::HttpResponse::Ok()
            .content_type("application/json")
            .body(json.to_string()),
        Err(e) => {
            log::error!("分页结果序列化失败: {}", e);
            actix_web::HttpResponse::InternalServerError()
                .body(format!("分页结果序列化失败: {}", e))
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
