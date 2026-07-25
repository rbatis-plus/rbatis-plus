/// ID generation strategy.
/// Mirrors MyBatis-Plus `IdType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdType {
    /// Assign by database auto-increment.
    Auto,
    /// Assign by input (user sets id).
    Input,
    /// Assign globally unique id.
    AssignId,
    /// Assign UUID string.
    AssignUuid,
    /// No assignment (default, id must be provided).
    None,
}

impl Default for IdType {
    fn default() -> Self { Self::None }
}
