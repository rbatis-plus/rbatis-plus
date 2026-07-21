#![forbid(unsafe_code)]
//! Deterministic code generation from an audited table schema.

use serde::{Deserialize, Serialize};
use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnSchema {
    pub name: String,
    pub rust_type: String,
    pub nullable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableSchema {
    pub table_name: String,
    pub struct_name: String,
    pub id_column: String,
    pub columns: Vec<ColumnSchema>,
}

pub fn generate_model(schema: &TableSchema) -> String {
    let mut output = format!(
        "#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, rbatis_plus::PlusModel)]\n#[rbatis_plus(table_name = \"{}\", id_column = \"{}\")]\npub struct {} {{\n",
        schema.table_name, schema.id_column, schema.struct_name
    );
    for column in &schema.columns {
        let ty = if column.nullable {
            format!("Option<{}>", column.rust_type)
        } else {
            column.rust_type.clone()
        };
        let _ = writeln!(output, "    pub {}: {},", column.name, ty);
    }
    output.push_str("}\n");
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generates_stable_model_metadata() {
        let output = generate_model(&TableSchema {
            table_name: "orders".into(),
            struct_name: "OrderPo".into(),
            id_column: "id".into(),
            columns: vec![
                ColumnSchema {
                    name: "id".into(),
                    rust_type: "i64".into(),
                    nullable: false,
                },
                ColumnSchema {
                    name: "note".into(),
                    rust_type: "String".into(),
                    nullable: true,
                },
            ],
        });
        assert!(output.contains("table_name = \"orders\""));
        assert!(output.contains("pub note: Option<String>"));
    }
}
