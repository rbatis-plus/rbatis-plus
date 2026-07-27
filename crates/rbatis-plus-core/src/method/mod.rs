//! SQL 注入方法（每个方法生成一条 SQL 模板）。
//!
//! 对应 Java：`mybatis-plus-core/src/main/java/com/baomidou/mybatisplus/core/injector/methods/` 包下 13 个文件。

mod abstract_method;
mod sql_method;
mod insert;
mod delete;
mod delete_by_id;
mod delete_by_ids;
mod update;
mod update_by_id;
mod select_by_id;
mod select_by_ids;
mod select_by_map;
mod select_count;
mod select_list;
mod select_maps;
mod select_one;
mod select_objs;
pub mod test_utils;

pub use abstract_method::{AbstractMethod, MethodResult};
pub use sql_method::SqlMethod;
pub use insert::Insert;
pub use delete::Delete;
pub use delete_by_id::DeleteById;
pub use delete_by_ids::DeleteByIds;
pub use update::Update;
pub use update_by_id::UpdateById;
pub use select_by_id::SelectById;
pub use select_by_ids::SelectByIds;
pub use select_by_map::SelectByMap;
pub use select_count::SelectCount;
pub use select_list::SelectList;
pub use select_maps::SelectMaps;
pub use select_one::SelectOne;
pub use select_objs::SelectObjs;
