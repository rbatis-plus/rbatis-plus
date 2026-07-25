// Source: mybatis-plus-core/.../conditions/AbstractWrapper.java

use super::merge_segments::MergeSegments;

/// The base wrapper shared by `QueryWrapper` and `UpdateWrapper`.
///
/// Mirrors Java `AbstractWrapper<T, R, Children>`.  In Rust, the "children"
/// pattern (self-type polymorphism) is handled by having all condition methods
/// take `&mut self` and return `&mut Self` or take `mut self` returning `Self`
/// depending on the API style.
///
/// Column type `R` is `String` for `QueryWrapper`/`UpdateWrapper` and a
/// closure / field-reference for Lambda variants (future).
#[derive(Debug, Clone, Default)]
pub struct AbstractWrapper {
    /// The accumulated WHERE-fragments and bind parameters.
    pub segments: MergeSegments,
    /// SQL comment (prepended).
    pub sql_comment: String,
    /// SQL first (prepended before comment).
    pub sql_first: String,
    /// SQL last (appended to the very end, e.g. `LIMIT 1`).
    pub sql_last: String,
    /// Entity instance (for entity-based conditions, optional).
    pub entity: Option<rbs::Value>,
}

impl AbstractWrapper {
    pub fn new() -> Self {
        Self::default()
    }

    // ── Raw fragment helpers ──────────────────────────────────────────

    /// Append a raw WHERE fragment (no param binding).
    pub fn add_fragment(&mut self, fragment: impl Into<String>) {
        self.segments.add_and(fragment);
    }

    /// Append a raw WHERE fragment with a bind parameter.
    pub fn add_fragment_param(&mut self, fragment: impl Into<String>, value: rbs::Value) {
        self.segments.add_and_param(fragment, value);
    }

    /// Append a raw WHERE fragment connected by OR (no param binding).
    pub fn add_fragment_or(&mut self, fragment: impl Into<String>) {
        self.segments.add_or(fragment);
    }

    /// Append a raw WHERE fragment with OR and a bind parameter.
    pub fn add_fragment_or_param(&mut self, fragment: impl Into<String>, value: rbs::Value) {
        self.segments.add_or_param(fragment, value);
    }

    // ── Condition helpers (called by trait methods) ───────────────────

    /// Build a single-equality fragment like `column = #{value}`.
    /// `column` is already the resolved DB column name.
    pub(crate) fn format_eq(column: &str, value: &rbs::Value) -> String {
        format!("{} = {}", column, Self::value_to_sql_literal(value))
    }

    /// Build `column <> value`.
    pub(crate) fn format_ne(column: &str, value: &rbs::Value) -> String {
        format!("{} <> {}", column, Self::value_to_sql_literal(value))
    }

    /// Build `column > value`.
    pub(crate) fn format_gt(column: &str, value: &rbs::Value) -> String {
        format!("{} > {}", column, Self::value_to_sql_literal(value))
    }

    /// Build `column >= value`.
    pub(crate) fn format_ge(column: &str, value: &rbs::Value) -> String {
        format!("{} >= {}", column, Self::value_to_sql_literal(value))
    }

    /// Build `column < value`.
    pub(crate) fn format_lt(column: &str, value: &rbs::Value) -> String {
        format!("{} < {}", column, Self::value_to_sql_literal(value))
    }

    /// Build `column <= value`.
    pub(crate) fn format_le(column: &str, value: &rbs::Value) -> String {
        format!("{} <= {}", column, Self::value_to_sql_literal(value))
    }

    /// Build `column LIKE '%value%'`.
    pub(crate) fn format_like(column: &str, value: &str) -> String {
        format!("{} LIKE '%{}%'", column, value)
    }

    /// Build `column LIKE '%value'` (left match).
    pub(crate) fn format_like_left(column: &str, value: &str) -> String {
        format!("{} LIKE '%{}'", column, value)
    }

    /// Build `column LIKE 'value%'` (right match).
    pub(crate) fn format_like_right(column: &str, value: &str) -> String {
        format!("{} LIKE '{}%'", column, value)
    }

    /// Build `column NOT LIKE '%value%'`.
    pub(crate) fn format_not_like(column: &str, value: &str) -> String {
        format!("{} NOT LIKE '%{}%'", column, value)
    }

    /// Build `column IS NULL`.
    pub(crate) fn format_is_null(column: &str) -> String {
        format!("{} IS NULL", column)
    }

    /// Build `column IS NOT NULL`.
    pub(crate) fn format_is_not_null(column: &str) -> String {
        format!("{} IS NOT NULL", column)
    }

    /// Build `column IN (v1, v2, ...)`.
    pub(crate) fn format_in(column: &str, values: &[rbs::Value]) -> String {
        let items: Vec<String> = values.iter().map(Self::value_to_sql_literal).collect();
        format!("{} IN ({})", column, items.join(", "))
    }

    /// Build `column NOT IN (v1, v2, ...)`.
    pub(crate) fn format_not_in(column: &str, values: &[rbs::Value]) -> String {
        let items: Vec<String> = values.iter().map(Self::value_to_sql_literal).collect();
        format!("{} NOT IN ({})", column, items.join(", "))
    }

    /// Build `column BETWEEN v1 AND v2`.
    pub(crate) fn format_between(column: &str, v1: &rbs::Value, v2: &rbs::Value) -> String {
        format!(
            "{} BETWEEN {} AND {}",
            column,
            Self::value_to_sql_literal(v1),
            Self::value_to_sql_literal(v2)
        )
    }

    /// Build `column NOT BETWEEN v1 AND v2`.
    pub(crate) fn format_not_between(column: &str, v1: &rbs::Value, v2: &rbs::Value) -> String {
        format!(
            "{} NOT BETWEEN {} AND {}",
            column,
            Self::value_to_sql_literal(v1),
            Self::value_to_sql_literal(v2)
        )
    }

    /// Convert an `rbs::Value` to an SQL literal string (quoted or not).
    fn value_to_sql_literal(value: &rbs::Value) -> String {
        match value {
            rbs::Value::Null => "NULL".to_string(),
            rbs::Value::Bool(b) => {
                if *b {
                    "1".to_string()
                } else {
                    "0".to_string()
                }
            }
            rbs::Value::I32(n) => n.to_string(),
            rbs::Value::I64(n) => n.to_string(),
            rbs::Value::U32(n) => n.to_string(),
            rbs::Value::U64(n) => n.to_string(),
            rbs::Value::F32(f) => f.to_string(),
            rbs::Value::F64(f) => f.to_string(),
            rbs::Value::String(s) => format!("'{}'", s.replace("'", "''")),
            rbs::Value::Binary(b) => format!("0x{}", hex::encode(b)),
            rbs::Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(Self::value_to_sql_literal).collect();
                items.join(", ")
            }
            _ => format!("'{}'", value.to_string()),
        }
    }

    // ── SQL segment assembly ──────────────────────────────────────────

    /// Build the complete WHERE clause (with `WHERE ` prefix if non-empty).
    pub fn build_where(&self) -> String {
        let seg = self.segments.sql_segment();
        if seg.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", seg)
        }
    }

    /// Return bind parameters (WHERE clause values).
    pub fn params(&self) -> &[rbs::Value] {
        self.segments.params()
    }

    /// Clear all conditions.
    pub fn clear(&mut self) {
        self.segments.clear();
        self.sql_comment.clear();
        self.sql_first.clear();
        self.sql_last.clear();
        self.entity = None;
    }
}
