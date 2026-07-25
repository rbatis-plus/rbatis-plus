// Source: rbatis-wrapper/src/wrapper.rs (Page struct, absorbed)
// Source: mybatis-plus-extension/.../pagination/IPage.java

use serde::{Deserialize, Serialize};

/// Paginated result wrapper.
///
/// Mirrors Java `com.baomidou.mybatisplus.extension.plugins.pagination.Page<T>`
/// and absorbs the `Page<T>` from `rbatis-wrapper`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    /// Query result data list.
    ///
    /// 查询数据列表
    pub records: Vec<T>,
    /// Total record count.
    ///
    /// 总数
    pub total: u64,
    /// Current page number (1-based).
    ///
    /// 当前页
    pub page_no: u64,
    /// Page size.
    ///
    /// 每页大小
    pub page_size: u64,
    /// Total page count (computed).
    ///
    /// 总页数
    pub pages: u64,
    /// Whether there is a next page.
    ///
    /// 是否有下一页
    pub has_next: bool,
}

impl<T> Page<T> {
    /// Create a new Page from records, total, page_no and page_size.
    ///
    /// Pages is computed as ceiling(total / page_size).
    /// Has_next is true when page_no < pages.
    pub fn new(records: Vec<T>, total: u64, page_no: u64, page_size: u64) -> Self {
        let pages = if page_size == 0 {
            0
        } else {
            (total + page_size - 1) / page_size
        };
        let has_next = page_no < pages;
        Self {
            records,
            total,
            page_no,
            page_size,
            pages,
            has_next,
        }
    }

    /// Create an empty page.
    pub fn empty(page_no: u64, page_size: u64) -> Self {
        Self::new(Vec::new(), 0, page_no, page_size)
    }
}

/// Page request parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRequest {
    /// Page number (1-based).
    pub page_no: u64,
    /// Page size.
    pub page_size: u64,
}

impl PageRequest {
    pub fn new(page_no: u64, page_size: u64) -> Self {
        Self {
            page_no: page_no.max(1),
            page_size: page_size.max(1),
        }
    }

    /// Compute the SQL OFFSET.
    pub fn offset(&self) -> u64 {
        (self.page_no.saturating_sub(1)) * self.page_size
    }
}

impl Default for PageRequest {
    fn default() -> Self {
        Self::new(1, 10)
    }
}
