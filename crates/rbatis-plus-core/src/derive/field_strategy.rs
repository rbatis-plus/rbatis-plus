/// Field strategy for determining when a field should be included in generated SQL.
/// Mirrors MyBatis-Plus `FieldStrategy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldStrategy {
    /// Always include the field.
    Always,
    /// Include only when the field value is not null.
    NotNull,
    /// Include when the field value is not null and not empty string.
    NotEmpty,
    /// Never include the field.
    Never,
    /// Use the default strategy from global config.
    Default,
}

impl Default for FieldStrategy {
    fn default() -> Self { Self::NotNull }
}
