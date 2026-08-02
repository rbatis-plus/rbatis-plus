//! SQL 方法名和模板字符串。
//!
//! 对应 Java：`com.baomidou.mybatisplus.core.enums.SqlMethod`（mybatis-plus-core）
//! 文件来源参考：`mybatis-plus-core/src/main/java/com/baomidou/mybatisplus/core/enums/SqlMethod.java`
//!
//! Java 3.5.17 的 SqlMethod 枚举包含 14 个 SQL 模板字符串，每个对应一个 Mapper 方法。
//! Rust 端保留相同结构，但用 Rust 字符串常量实现（不依赖 MyBatis 的 SqlMethod 枚举）。

/// SQL 方法模板字符串（对标 Java `SqlMethod` 枚举）。
///
/// 每个变体对应一个 Mapper CRUD 方法的 SQL 模板。
/// Java 端通过 `SqlMethod.INSERT_ONE.format(tableName, columnScript, valuesScript)` 生成最终 SQL。
/// Rust 端直接在 `Method::generate_sql(&self, table_info: &TableInfo) -> String` 中拼装。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SqlMethod {
    /// INSERT INTO <table> (<columns>) VALUES (<values>)
    InsertOne,
    /// DELETE FROM <table> WHERE <condition>
    Delete,
    /// UPDATE <table> SET <set> <where>
    Update,
    /// SELECT COUNT(*) FROM <table> <where>
    SelectCount,
    /// SELECT <columns> FROM <table> <where>
    SelectList,
    /// SELECT <columns> FROM <table> <where> WHERE id IN (<ids>)
    SelectByIds,
    /// SELECT <columns> FROM <table> WHERE <pk> = #{id}
    SelectById,
    /// SELECT <columns> FROM <table> <where> — 返回 Map
    SelectMaps,
    /// SELECT <columns> FROM <table> <where> — 返回单行首值
    SelectObjs,
    /// SELECT <columns> FROM <table> <where> — 返回 Map
    SelectByMap,
    /// SELECT <columns> FROM <table> WHERE <condition>
    SelectOne,
    /// DELETE FROM <table> WHERE <pk> IN (<ids>)
    DeleteByIds,
    /// UPDATE <table> SET <set> WHERE <pk> = #{id} <version>
    UpdateById,
    /// SELECT <columns> FROM <table> WHERE <condition> <cursor>
    SelectWithCursor,
    /// 逻辑删除 UPDATE：<table> SET deleted=1 WHERE <condition>
    LogicDelete,
    /// 逻辑删除 UPDATE：<table> SET <version>+1, deleted=1 WHERE <condition>
    LogicDeleteById,
}

impl SqlMethod {
    /// 返回对应的方法名（用于 Mapper 方法调用）。
    pub fn method_name(&self) -> &'static str {
        match self {
            Self::InsertOne      => "insert",
            Self::Delete         => "delete",
            Self::Update         => "update",
            Self::SelectCount    => "selectCount",
            Self::SelectList     => "selectList",
            Self::SelectByIds    => "selectByIds",
            Self::SelectById     => "selectById",
            Self::SelectMaps     => "selectMaps",
            Self::SelectObjs     => "selectObjs",
            Self::SelectByMap    => "selectByMap",
            Self::SelectOne      => "selectOne",
            Self::DeleteByIds    => "deleteByIds",
            Self::UpdateById     => "updateById",
            Self::SelectWithCursor => "selectWithCursor",
            Self::LogicDelete    => "delete",
            Self::LogicDeleteById => "deleteById",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_name_matches_java() {
        assert_eq!(SqlMethod::InsertOne.method_name(), "insert");
        assert_eq!(SqlMethod::Delete.method_name(), "delete");
        assert_eq!(SqlMethod::SelectList.method_name(), "selectList");
        assert_eq!(SqlMethod::UpdateById.method_name(), "updateById");
    }
}
