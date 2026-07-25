/// Auto-fill strategy for insert/update operations.
/// Mirrors MyBatis-Plus `FieldFill`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldFill {
    /// Default, never auto-fill.
    Default,
    /// Auto-fill on insert.
    Insert,
    /// Auto-fill on update.
    Update,
    /// Auto-fill on insert and update.
    InsertUpdate,
}

impl Default for FieldFill {
    fn default() -> Self { Self::Default }
}
