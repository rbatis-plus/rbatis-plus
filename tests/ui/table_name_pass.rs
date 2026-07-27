// trybuild 正例：完整的 #[derive(TableName)] 应该编译成功并生成4个 trait impl + 列访问器。

use rbatis_plus_macros::TableName;  // derive 宏
use rbatis_plus::core::derive::{TableId, Version, TableLogic, IdType};
use rbatis_plus::core::derive::TableName as TableNameTrait;  // trait

// 完整注解：表名 + 主键(auto) + 乐观锁 + 逻辑删除
#[derive(Debug, Clone, TableName)]
#[table_name = "sys_user"]
pub struct SysUser {
    #[table_id(type = "auto")]
    pub id: u64,
    pub name: String,
    #[version]
    pub version: u64,
    #[table_logic(value = "1", not_value = "0")]
    pub deleted: i32,
    pub email: String,
}

fn main() {
    // 验证 TableName trait
    assert_eq!(SysUser::table_name(), "sys_user");
    assert_eq!(<SysUser as TableNameTrait>::table_name(), "sys_user");

    // 验证 TableId impl
    assert_eq!(SysUser::id_type(), IdType::Auto);
    assert_eq!(SysUser::id_column(), "id");

    // 验证 Version impl
    assert_eq!(SysUser::version_column(), "version");

    // 验证 TableLogic impl
    assert_eq!(SysUser::logic_column(), "deleted");
    assert_eq!(SysUser::logic_delete_value(), Some("1"));
    assert_eq!(SysUser::logic_not_delete_value(), Some("0"));

    // 验证列访问器生成（Type-safe Column<F>）
    let _name_col: rbatis_plus::core::conditions::query::Column<String> = SysUser::column_name();
    let _id_col: rbatis_plus::core::conditions::query::Column<u64> = SysUser::column_id();

    // 验证列常量
    assert_eq!(SysUser::COLUMN_NAME, "name");
    assert_eq!(SysUser::COLUMN_ID, "id");
    assert_eq!(SysUser::COLUMN_VERSION, "version");
    assert_eq!(SysUser::COLUMN_EMAIL, "email");

    println!("All derive(TableName) assertions passed!");
}
