use super::id_type::IdType;

/// Marker trait for primary key fields.
/// Derive with `#[derive(TableId)]` macro.
pub trait TableId {
    fn id_type() -> IdType { IdType::None }
    fn id_column() -> &'static str { "id" }
}
