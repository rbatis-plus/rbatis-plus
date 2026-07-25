use super::field_fill::FieldFill;
use super::field_strategy::FieldStrategy;

#[derive(Debug, Clone)]
pub struct TableFieldAttr {
    pub property: &'static str,
    pub column: &'static str,
    pub exist: bool,
    pub where_strategy: FieldStrategy,
    pub insert_strategy: FieldStrategy,
    pub update_strategy: FieldStrategy,
    pub fill: FieldFill,
    pub select: bool,
    pub version: bool,
    pub logic_delete: bool,
    pub logic_not_delete_value: &'static str,
    pub logic_delete_value: &'static str,
    pub update: &'static str,
}

impl Default for TableFieldAttr {
    fn default() -> Self {
        Self {
            property: "",
            column: "",
            exist: true,
            where_strategy: FieldStrategy::default(),
            insert_strategy: FieldStrategy::default(),
            update_strategy: FieldStrategy::default(),
            fill: FieldFill::default(),
            select: true,
            version: false,
            logic_delete: false,
            logic_not_delete_value: "",
            logic_delete_value: "",
            update: "",
        }
    }
}
