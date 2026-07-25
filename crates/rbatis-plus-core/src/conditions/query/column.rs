// Source: mybatis-plus-core/.../toolkit/support/ColumnCache.java
// Source: mybatis-plus-core/.../conditions/AbstractLambdaWrapper.java

use std::marker::PhantomData;

/// 类型安全的列引用（对标 Java `SFunction<T, ?>` 的编译期列解析）。
///
/// Java 中 `LambdaQueryWrapper` 通过 `User::getName` 序列化 lambda 解析列名；
/// Rust 没有运行时 lambda 反射，改用 `Column<F>` + derive 宏在编译期生成列名。
///
/// `F` 是字段的 Rust 类型（如 `String`、`i64`），用于编译期类型检查。
///
/// # 用法
///
/// 通常由 `#[derive(TableName)]` 宏自动生成，不需要手动构造：
///
/// ```ignore
/// use rbatis_plus::core::conditions::query::lambda_query_wrapper::Column;
///
/// // derive 宏会生成：User::column_name() -> Column<String>
/// let col: Column<String> = Column::new("user_name");
/// assert_eq!(col.name(), "user_name");
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Column<F> {
    /// 数据库列名（snake_case），如 `"user_name"`。
    name: &'static str,
    /// 字段 Rust 类型的幽灵标记，用于编译期类型安全。
    _phantom: PhantomData<F>,
}

impl<F> Column<F> {
    /// 创建列引用。
    ///
    /// 通常由 derive 宏调用，用户无需手动构造。
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _phantom: PhantomData,
        }
    }

    /// 获取数据库列名。
    pub fn name(&self) -> &'static str {
        self.name
    }
}

/// 实体类的列元数据（对标 Java `TableInfo` 中的列映射缓存）。
///
/// 由 `#[derive(TableName)]` 宏自动生成，为每个实体类型提供列名常量。
///
/// # 用法
///
/// ```ignore
/// #[derive(TableName)]
/// struct User {
///     #[table_id]
///     id: i64,
///     name: String,
/// }
///
/// // derive 宏生成 LambdaColumns 实现
/// let name_col = User::column_name(); // Column<String>
/// assert_eq!(name_col.name(), "name");
/// ```
pub trait LambdaColumns {
    /// 通过闭包获取列引用（运行时回退方案）。
    ///
    /// 当无法使用 derive 宏时，可通过 `column_of("field_name")` 手动查找。
    fn column_of(field_name: &str) -> Option<&'static str>;
}
