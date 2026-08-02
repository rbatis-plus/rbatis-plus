//! 数据源配置（对标 mybatis-plus-generator `DataSourceConfig`）。

use serde::{Deserialize, Serialize};

/// 数据源配置（对标 Java `com.baomidou.mybatisplus.generator.config.DataSourceConfig`）。
///
/// 指定数据库连接信息，用于查询表元数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceConfig {
    /// JDBC/数据库连接 URL。
    pub url: String,
    /// 用户名。
    pub username: String,
    /// 密码。
    pub password: String,
    /// 数据库类型（MySQL/PostgreSQL/SQLite 等）。
    pub db_type: DbType,
    /// schema 名（可选）。
    pub schema: Option<String>,
}

/// 支持的数据库类型（对标 Java `com.baomidou.mybatisplus.annotation.DbType`）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DbType {
    MySql,
    PostgreSql,
    Sqlite,
    Oracle,
    SqlServer,
    MariaDb,
    ClickHouse,
    Other(String),
}

impl Default for DataSourceConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            username: String::new(),
            password: String::new(),
            db_type: DbType::MySql,
            schema: None,
        }
    }
}

impl DataSourceConfig {
    /// 从 URL 自动检测数据库类型。
    pub fn detect_db_type(url: &str) -> DbType {
        let lower = url.to_lowercase();
        if lower.starts_with("mysql") || lower.contains("mysql") {
            DbType::MySql
        } else if lower.starts_with("postgres") || lower.contains("postgresql") {
            DbType::PostgreSql
        } else if lower.starts_with("sqlite") || lower.contains("sqlite") {
            DbType::Sqlite
        } else if lower.starts_with("oracle") || lower.contains("oracle") {
            DbType::Oracle
        } else if lower.starts_with("sqlserver") || lower.contains("sqlserver") {
            DbType::SqlServer
        } else if lower.starts_with("mariadb") || lower.contains("mariadb") {
            DbType::MariaDb
        } else if lower.starts_with("clickhouse") || lower.contains("clickhouse") {
            DbType::ClickHouse
        } else {
            DbType::Other("unknown".to_string())
        }
    }

    /// 创建构建器。
    pub fn builder() -> DataSourceConfigBuilder {
        DataSourceConfigBuilder::default()
    }
}

/// DataSourceConfig 构建器。
#[derive(Default)]
pub struct DataSourceConfigBuilder {
    config: DataSourceConfig,
}

impl DataSourceConfigBuilder {
    pub fn url(mut self, url: impl Into<String>) -> Self {
        let url_str = url.into();
        self.config.db_type = DataSourceConfig::detect_db_type(&url_str);
        self.config.url = url_str;
        self
    }
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.config.username = username.into();
        self
    }
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.config.password = password.into();
        self
    }
    pub fn db_type(mut self, db_type: DbType) -> Self {
        self.config.db_type = db_type;
        self
    }
    pub fn schema(mut self, schema: impl Into<String>) -> Self {
        self.config.schema = Some(schema.into());
        self
    }
    pub fn build(self) -> DataSourceConfig {
        self.config
    }
}
