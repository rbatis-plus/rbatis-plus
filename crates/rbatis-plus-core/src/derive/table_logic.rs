/// Marker trait for logical delete fields.
/// Derive with `#[derive(TableLogic)]` macro.
pub trait TableLogic {
    fn logic_column() -> &'static str;
    fn logic_value() -> &'static str { "0" }
    fn not_logic_value() -> &'static str { "1" }
}
