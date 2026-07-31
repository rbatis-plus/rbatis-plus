// Source: mybatis-plus-core/.../conditions/segments/MergeSegments.java
// Source: mybatis-plus-core/.../conditions/segments/NormalSegmentList.java

use rbs::Value;

/// 单条 WHERE 片段，携带其连接词（AND / OR）。
/// 存储的是不含连接词的裸片段，连接词在 `sql_segment()` 拼接。
#[derive(Debug, Clone)]
struct Segment {
    /// `AND` 或 `OR`，首条片段固定为 `AND`（拼接时跳过）。
    connector: &'static str,
    /// 不含前导连接词的 SQL 片段，例如 `name = 'Alice'`。
    fragment: String,
}

/// 累积 SQL WHERE 片段及对应绑定参数。
///
/// 对标 MyBatis-Plus `MergeSegments` + `NormalSegmentList` + `paramNameValuePairs`。
/// 每次条件方法调用会追加一个 `Segment`，`sql_segment()` 最终拼接成完整 WHERE。
#[derive(Debug, Clone, Default)]
pub struct MergeSegments {
    /// 有序片段列表（每条带连接词）。
    segments: Vec<Segment>,
    /// 绑定参数（顺序与片段顺序一致）。
    params: Vec<Value>,
    /// 参数名序号。
    param_seq: usize,
    /// 下一个追加的片段使用 OR 连接。
    pending_or: bool,
}

impl MergeSegments {
    pub fn new() -> Self {
        Self::default()
    }

    /// 生成下一个参数名（`MPGENVAL{n}`）。
    ///
    /// 预留：与 MyBatis-Plus `paramNameValuePairs` 的 `MPGENVAL` 命名对应，
    /// 当前参数按位置绑定（`params()`），命名参数生成后续启用。
    #[allow(dead_code)]
    fn next_param_name(&mut self) -> String {
        self.param_seq += 1;
        format!("MPGENVAL{}", self.param_seq)
    }

    /// 追加 AND 片段（裸片段，不含 `AND` 前缀）。
    pub fn add_and(&mut self, fragment: impl Into<String>) {
        self.push_segment("AND", fragment.into(), None);
    }

    /// 追加 OR 片段（裸片段，不含 `OR` 前缀）。
    pub fn add_or(&mut self, fragment: impl Into<String>) {
        self.push_segment("OR", fragment.into(), None);
    }

    /// 追加带参数的 AND 片段。
    pub fn add_and_param(&mut self, fragment: impl Into<String>, value: Value) {
        self.push_segment("AND", fragment.into(), Some(value));
    }

    /// 追加带参数的 OR 片段。
    pub fn add_or_param(&mut self, fragment: impl Into<String>, value: Value) {
        self.push_segment("OR", fragment.into(), Some(value));
    }

    /// 标记下一个追加的片段使用 OR 连接。
    pub fn set_next_or(&mut self, on: bool) {
        self.pending_or = on;
    }

    fn push_segment(&mut self, connector: &'static str, fragment: String, value: Option<Value>) {
        let effective = if self.pending_or { "OR" } else { connector };
        self.segments.push(Segment { connector: effective, fragment });
        if let Some(v) = value {
            self.params.push(v);
        }
        self.pending_or = false;
    }

    /// 拼接完整 WHERE 子句（不含 `WHERE` 关键字）。
    ///
    /// 首条片段不带连接词；后续片段用其 `connector` 拼接。
    pub fn sql_segment(&self) -> String {
        if self.segments.is_empty() {
            return String::new();
        }
        let mut result = String::new();
        for seg in &self.segments {
            if !result.is_empty() {
                result.push(' ');
                result.push_str(seg.connector);
                result.push(' ');
            }
            result.push_str(&seg.fragment);
        }
        result
    }

    /// 返回按顺序收集的绑定参数。
    pub fn params(&self) -> &[Value] {
        &self.params
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// 清空所有片段和参数。
    pub fn clear(&mut self) {
        self.segments.clear();
        self.params.clear();
        self.pending_or = false;
        self.param_seq = 0;
    }
}
