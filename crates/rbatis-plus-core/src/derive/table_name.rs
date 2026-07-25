/// Marker trait for entities that map to a database table.
/// Derive with `#[derive(TableName)]` macro once rbatis-plus-macros is ready.
/// Default impl returns the struct name converted to snake_case.
pub trait TableName {
    fn table_name() -> &'static str;
}
