//! Derive trait definitions for RBatis-Plus.
//!
//! These traits are the Rust equivalents of MyBatis-Plus Java annotations.
//! They are implemented automatically by `#[derive(TableName)]` and friends
//! from `rbatis-plus-macros`.

pub mod encrypted_field;
pub mod encrypted_table;
pub mod field_fill;
pub mod field_strategy;
pub mod i18n_column;
pub mod id_type;
pub mod signature_field;
pub mod table_field;
pub mod table_id;
pub mod table_logic;
pub mod table_name;
pub mod table_signature;
pub mod version;

// Re-exports: 每个模块仅暴露对外需要的类型（禁止 wildcard）
pub use encrypted_field::EncryptedFieldAttr;
pub use encrypted_table::EncryptedTable;
pub use field_fill::FieldFill;
pub use field_strategy::FieldStrategy;
pub use i18n_column::I18nColumnAttr;
pub use i18n_column::I18nColumn;
pub use id_type::IdType;
pub use signature_field::SignatureFieldAttr;
pub use table_field::TableFieldAttr;
pub use table_id::TableId;
pub use table_logic::TableLogic;
pub use table_name::{TableName, TableNameInfo};
pub use table_signature::TableSignature;
pub use version::Version;
