// trybuild 反例：proc-macro derive(TableName) 只支持 struct，不支持 enum。
// 期望的编译错误：derive 只能用于 struct

use rbatis_plus_macros::TableName;

#[derive(TableName)]
#[table_name = "empty_table"]
pub enum NotAStruct {
    Variant1,
    Variant2,
}

fn main() {}
