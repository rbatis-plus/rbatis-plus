// Source: mybatis-plus-core/.../conditions/interfaces/Compare.java

use super::abstract_wrapper::AbstractWrapper;
use rbs::Value;

/// Comparison condition methods.
///
/// Mirrors Java `com.baomidou.mybatisplus.core.conditions.interfaces.Compare<R>`.
/// Every method has an implicit `condition = true` — for conditional
/// versions use the `if_*` variants.
pub trait Compare {
    /// `column = value`
    ///
    /// 等于 =
    fn eq(&mut self, column: &str, value: impl Into<Value>) -> &mut Self;

    /// Conditional eq — only applies when `condition` is true.
    fn if_eq(&mut self, condition: bool, column: &str, value: impl Into<Value>) -> &mut Self;

    /// `column <> value` (or `IS NULL` if value is null)
    ///
    /// 不等于 &lt;&gt;
    fn ne(&mut self, column: &str, value: impl Into<Value>) -> &mut Self;
    fn if_ne(&mut self, condition: bool, column: &str, value: impl Into<Value>) -> &mut Self;

    /// `column > value`
    ///
    /// 大于 &gt;
    fn gt(&mut self, column: &str, value: impl Into<Value>) -> &mut Self;
    fn if_gt(&mut self, condition: bool, column: &str, value: impl Into<Value>) -> &mut Self;

    /// `column >= value`
    ///
    /// 大于等于 &gt;=
    fn ge(&mut self, column: &str, value: impl Into<Value>) -> &mut Self;
    fn if_ge(&mut self, condition: bool, column: &str, value: impl Into<Value>) -> &mut Self;

    /// `column < value`
    ///
    /// 小于 &lt;
    fn lt(&mut self, column: &str, value: impl Into<Value>) -> &mut Self;
    fn if_lt(&mut self, condition: bool, column: &str, value: impl Into<Value>) -> &mut Self;

    /// `column <= value`
    ///
    /// 小于等于 &lt;=
    fn le(&mut self, column: &str, value: impl Into<Value>) -> &mut Self;
    fn if_le(&mut self, condition: bool, column: &str, value: impl Into<Value>) -> &mut Self;

    /// `column BETWEEN val1 AND val2`
    ///
    /// BETWEEN 值1 AND 值2
    fn between(&mut self, column: &str, val1: impl Into<Value>, val2: impl Into<Value>) -> &mut Self;
    fn if_between(
        &mut self,
        condition: bool,
        column: &str,
        val1: impl Into<Value>,
        val2: impl Into<Value>,
    ) -> &mut Self;

    /// `column NOT BETWEEN val1 AND val2`
    ///
    /// NOT BETWEEN 值1 AND 值2
    fn not_between(
        &mut self,
        column: &str,
        val1: impl Into<Value>,
        val2: impl Into<Value>,
    ) -> &mut Self;
    fn if_not_between(
        &mut self,
        condition: bool,
        column: &str,
        val1: impl Into<Value>,
        val2: impl Into<Value>,
    ) -> &mut Self;

    /// `column = value OR column IS NULL` (when value is null, just IS NULL)
    ///
    /// 等于 = （当 value 为 null 时变为 IS NULL）
    fn eq_or_is_null(&mut self, column: &str, value: impl Into<Value>) -> &mut Self;
}

// ── Blanket impl for AbstractWrapper ───────────────────────────────────

impl Compare for AbstractWrapper {
    fn eq(&mut self, column: &str, value: impl Into<Value>) -> &mut Self {
        self.if_eq(true, column, value)
    }
    fn if_eq(&mut self, condition: bool, column: &str, value: impl Into<Value>) -> &mut Self {
        if condition {
            let v = value.into();
            self.add_fragment(Self::format_eq(column, &v));
        }
        self
    }

    fn ne(&mut self, column: &str, value: impl Into<Value>) -> &mut Self {
        self.if_ne(true, column, value)
    }
    fn if_ne(&mut self, condition: bool, column: &str, value: impl Into<Value>) -> &mut Self {
        if condition {
            let v = value.into();
            self.add_fragment(Self::format_ne(column, &v));
        }
        self
    }

    fn gt(&mut self, column: &str, value: impl Into<Value>) -> &mut Self {
        self.if_gt(true, column, value)
    }
    fn if_gt(&mut self, condition: bool, column: &str, value: impl Into<Value>) -> &mut Self {
        if condition {
            let v = value.into();
            self.add_fragment(Self::format_gt(column, &v));
        }
        self
    }

    fn ge(&mut self, column: &str, value: impl Into<Value>) -> &mut Self {
        self.if_ge(true, column, value)
    }
    fn if_ge(&mut self, condition: bool, column: &str, value: impl Into<Value>) -> &mut Self {
        if condition {
            let v = value.into();
            self.add_fragment(Self::format_ge(column, &v));
        }
        self
    }

    fn lt(&mut self, column: &str, value: impl Into<Value>) -> &mut Self {
        self.if_lt(true, column, value)
    }
    fn if_lt(&mut self, condition: bool, column: &str, value: impl Into<Value>) -> &mut Self {
        if condition {
            let v = value.into();
            self.add_fragment(Self::format_lt(column, &v));
        }
        self
    }

    fn le(&mut self, column: &str, value: impl Into<Value>) -> &mut Self {
        self.if_le(true, column, value)
    }
    fn if_le(&mut self, condition: bool, column: &str, value: impl Into<Value>) -> &mut Self {
        if condition {
            let v = value.into();
            self.add_fragment(Self::format_le(column, &v));
        }
        self
    }

    fn between(
        &mut self,
        column: &str,
        val1: impl Into<Value>,
        val2: impl Into<Value>,
    ) -> &mut Self {
        self.if_between(true, column, val1, val2)
    }
    fn if_between(
        &mut self,
        condition: bool,
        column: &str,
        val1: impl Into<Value>,
        val2: impl Into<Value>,
    ) -> &mut Self {
        if condition {
            let v1 = val1.into();
            let v2 = val2.into();
            self.add_fragment(Self::format_between(column, &v1, &v2));
        }
        self
    }

    fn not_between(
        &mut self,
        column: &str,
        val1: impl Into<Value>,
        val2: impl Into<Value>,
    ) -> &mut Self {
        self.if_not_between(true, column, val1, val2)
    }
    fn if_not_between(
        &mut self,
        condition: bool,
        column: &str,
        val1: impl Into<Value>,
        val2: impl Into<Value>,
    ) -> &mut Self {
        if condition {
            let v1 = val1.into();
            let v2 = val2.into();
            self.add_fragment(Self::format_not_between(column, &v1, &v2));
        }
        self
    }

    fn eq_or_is_null(&mut self, column: &str, value: impl Into<Value>) -> &mut Self {
        let v = value.into();
        if matches!(v, Value::Null) {
            self.add_fragment(Self::format_is_null(column));
        } else {
            self.add_fragment(Self::format_eq(column, &v));
        }
        self
    }
}
