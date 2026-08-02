/// 表名映射 trait — 实体到数据库表的绑定关系。
///
/// 对应 Java：`com.baomidou.mybatisplus.annotation.TableName`（mybatis-plus-annotation）
/// 文件来源参考：`mybatis-plus-annotation/src/main/java/com/baomidou/mybatisplus/annotation/TableName.java`
///
/// 由 `#[derive(TableName)]` 宏自动生成实现。默认 `table_name()` 返回结构体名的 snake_case 形式。
///
/// ```rust
/// use rbatis_plus_core::TableName;
///
/// trait UserTableName {
///     // derive 宏自动实现
/// }
///
/// // derive(TableName) 生成的实现示例（假设结构体名为 User）：
/// // impl TableName for User {
/// //     fn table_name() -> &'static str { "user" }
/// //     fn schema() -> Option<&'static str> { None }
/// //     fn keep_global_prefix() -> bool { false }
/// //     fn result_map() -> Option<&'static str> { None }
/// //     fn auto_result_map() -> bool { false }
/// //     fn include_properties() -> &'static [&'static str] { &[] }
/// //     fn exclude_properties() -> &'static [&'static str] { &[] }
/// // }
/// ```
pub trait TableName {
    /// 实体对应的表名（对应 Java `@TableName.value()`；默认空串表示由结构体名 snake_case 推导）。
    fn table_name() -> &'static str;

    /// 数据库 schema 名（对应 Java `@TableName.schema()`；默认空串表示由全局配置推导）。
    fn schema() -> Option<&'static str> {
        None
    }

    /// 是否保持使用全局的 tablePrefix（对应 Java `@TableName.keepGlobalPrefix()`；默认 false）。
    fn keep_global_prefix() -> bool {
        false
    }

    /// 实体映射 ResultMap 名（对应 Java `@TableName.resultMap()`；默认空串表示不指定）。
    fn result_map() -> Option<&'static str> {
        None
    }

    /// 是否自动构建 resultMap（对应 Java `@TableName.autoResultMap()`；默认 false）。
    fn auto_result_map() -> bool {
        false
    }

    /// 只需要的属性名列表（对应 Java `@TableName.properties()`；默认空切片）。
    fn include_properties() -> &'static [&'static str] {
        &[]
    }

    /// 需要排除的属性名列表（对应 Java `@TableName.excludeProperty()`；默认空切片）。
    fn exclude_properties() -> &'static [&'static str] {
        &[]
    }
}

/// 运行时表名元数据结构体（由 derive 宏填充，供 `MetaObjectHandler` / `TableInfoHelper` 使用）。
///
/// 对应 Java：`TableInfo` 中由 `@TableName` 注解填充的字段集合。
/// Rust 端用独立结构体保存，因为 `trait` 方法返回值的表达力有限。
#[derive(Debug, Clone)]
pub struct TableNameInfo {
    /// 数据库表名（对应 Java `TableInfo.tableName`）。
    pub table_name: String,
    /// 数据库 schema（对应 Java `TableInfo.currentSchema`）。
    pub schema: Option<String>,
    /// 是否保持全局 tablePrefix（对应 Java `TableInfo.keepGlobalPrefix`）。
    pub keep_global_prefix: bool,
    /// ResultMap 名（对应 Java `TableInfo.resultMap`）。
    pub result_map: Option<String>,
    /// 是否自动构建 resultMap（对应 Java `TableInfo.autoResultMap`）。
    pub auto_result_map: bool,
    /// 包含的属性名（对应 Java `TableInfo.properties`；优先于 exclude）。
    pub include_properties: Vec<String>,
    /// 排除的属性名（对应 Java `TableInfo.excludeProperty`）。
    pub exclude_properties: Vec<String>,
}

impl TableNameInfo {
    /// 从 `TableName` trait 方法构建（derive 宏展开时调用）。
    pub fn from_trait<T: TableName>() -> Self {
        Self {
            table_name: T::table_name().to_string(),
            schema: T::schema().map(|s| s.to_string()),
            keep_global_prefix: T::keep_global_prefix(),
            result_map: T::result_map().map(|s| s.to_string()),
            auto_result_map: T::auto_result_map(),
            include_properties: T::include_properties().iter().map(|s| s.to_string()).collect(),
            exclude_properties: T::exclude_properties().iter().map(|s| s.to_string()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockUser;
    impl TableName for MockUser {
        fn table_name() -> &'static str { "user" }
        fn schema() -> Option<&'static str> { Some("public") }
        fn keep_global_prefix() -> bool { true }
        fn result_map() -> Option<&'static str> { Some("UserRM") }
        fn auto_result_map() -> bool { true }
        fn include_properties() -> &'static [&'static str] { &["name", "email"] }
        fn exclude_properties() -> &'static [&'static str] { &["password"] }
    }

    #[test]
    fn trait_methods_match_java_semantics() {
        assert_eq!(MockUser::table_name(), "user");
        assert_eq!(MockUser::schema(), Some("public"));
        assert!(MockUser::keep_global_prefix());
        assert_eq!(MockUser::result_map(), Some("UserRM"));
        assert!(MockUser::auto_result_map());
        assert_eq!(MockUser::include_properties(), &["name" as &str, "email"]);
        assert_eq!(MockUser::exclude_properties(), &["password" as &str]);
    }

    #[test]
    fn table_name_info_from_trait() {
        let info = TableNameInfo::from_trait::<MockUser>();
        assert_eq!(info.table_name, "user");
        assert_eq!(info.schema, Some("public".to_string()));
        assert!(info.keep_global_prefix);
        assert_eq!(info.result_map, Some("UserRM".to_string()));
        assert!(info.auto_result_map);
        assert_eq!(info.include_properties, vec!["name".to_string(), "email".to_string()]);
        assert_eq!(info.exclude_properties, vec!["password".to_string()]);
    }

    #[test]
    fn default_trait_returns_defaults() {
        struct DefaultImpl;
        impl TableName for DefaultImpl {
            fn table_name() -> &'static str { "default_impl" }
        }

        assert_eq!(DefaultImpl::table_name(), "default_impl");
        assert_eq!(DefaultImpl::schema(), None);
        assert!(!DefaultImpl::keep_global_prefix());
        assert_eq!(DefaultImpl::result_map(), None);
        assert!(!DefaultImpl::auto_result_map());
        assert_eq!(DefaultImpl::include_properties(), &[] as &[&str]);
        assert_eq!(DefaultImpl::exclude_properties(), &[] as &[&str]);
    }
}
