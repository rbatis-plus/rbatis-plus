//! SQL 语句类型枚举。
//!
//! 对应 Java：`com.baomidou.mybatisplus.extension.parser.IJSqlParser` 中的语句类型识别逻辑。
//!
//! mybatis-plus-jsqlparser 中，StatementType 用于在分页插件中区分 SELECT / INSERT / UPDATE / DELETE。
//!
//! 文件来源参考：`mybatis-plus-jsqlparser-support/.../plugins/inner/PaginationInnerInterceptor.java`

/// SQL 语句类型（对标 Java `StatementType`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatementType {
    /// SELECT 查询
    Select,
    /// INSERT 插入
    Insert,
    /// UPDATE 更新
    Update,
    /// DELETE 删除
    Delete,
    /// 其他（DDL 等）
    Other(String),
}
