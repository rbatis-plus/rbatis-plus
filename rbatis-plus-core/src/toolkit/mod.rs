//! Toolkit utilities for RBatis-Plus.
//!
//! Mirrors `mybatis-plus-core/.../toolkit/`.

/// SQL utility functions.
pub mod sql_utils {
    /// Quote a string value for SQL (single-quote, escape internal quotes).
    pub fn quote_string(s: &str) -> String {
        format!("'{}'", s.replace("'", "''"))
    }

    /// Convert a camelCase or PascalCase identifier to snake_case.
    pub fn camel_to_snake(s: &str) -> String {
        let mut result = String::with_capacity(s.len() + 4);
        for (i, ch) in s.chars().enumerate() {
            if ch.is_uppercase() {
                if i > 0 {
                    result.push('_');
                }
                result.extend(ch.to_lowercase());
            } else {
                result.push(ch);
            }
        }
        result
    }
}

/// Common constants.
pub mod constants {
    /// The default alias for entity parameters in MyBatis-Plus SQL templates.
    pub const ENTITY_ALIAS: &str = "et";

    /// The default wrapper parameter alias.
    pub const WRAPPER_ALIAS: &str = "ew";

    /// The default param-name-value-pairs key prefix.
    pub const PARAM_NAME_VALUE_PAIRS: &str = "paramNameValuePairs";

    /// Generated param-name prefix.
    pub const MP_GEN_VAL: &str = "MPGENVAL";
}
