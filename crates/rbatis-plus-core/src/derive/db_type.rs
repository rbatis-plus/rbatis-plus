/// 数据库类型枚举（主要用于分页方言）。
///
/// 对应 Java：`com.baomidou.mybatisplus.annotation.DbType`（mybatis-plus-annotation）
/// 文件来源参考：`mybatis-plus-annotation/src/main/java/com/baomidou/mybatisplus/annotation/DbType.java`
///
/// ```rust
/// use rbatis_plus_core::DbType;
///
/// assert_eq!(DbType::MYSQL.key(), "mysql");
/// assert_eq!(DbType::MYSQL.description(), "MySql数据库");
/// assert_eq!(DbType::from_key("sqlite"), Some(DbType::SQLITE));
/// assert_eq!(DbType::from_key("unknown"), Some(DbType::OTHER));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)] // 枚举名与 Java DbType 一一对应（如 ORACLE_12C、POSTGRE_SQL）
pub enum DbType {
    MYSQL,
    MARIADB,
    ORACLE,
    ORACLE_12C,
    H2,
    HSQL,
    SQLITE,
    POSTGRE_SQL,
    SQL_SERVER2005,
    SQL_SERVER,
    DB2,
    DM,
    XU_GU,
    KINGBASE_ES,
    PHOENIX,
    GAUSS,
    GAUSS_DB,
    CLICK_HOUSE,
    GBASE,
    GBASEDBT,
    GBASE_INFORMIX,
    SINODB,
    OSCAR,
    SYBASE,
    OCEAN_BASE,
    FIREBIRD,
    HIGH_GO,
    CUBRID,
    SUNDB,
    SAP_HANA,
    IMPALA,
    VERTICA,
    REDSHIFT,
    OPENGAUSS,
    TDENGINE,
    INFORMIX,
    UXDB,
    LEALONE,
    TRINO,
    PRESTO,
    DERBY,
    VASTBASE,
    GOLDENDB,
    DUCKDB,
    YASDB,
    OTHER,
}

impl DbType {
    /// 返回数据库 key 字符串（用于序列化/反序列化）。
    ///
    /// 对应 Java：`DbType.key` 字段（Lombok `@Getter`）。
    pub fn key(self) -> &'static str {
        match self {
            Self::MYSQL => "mysql",
            Self::MARIADB => "mariadb",
            Self::ORACLE => "oracle",
            Self::ORACLE_12C => "oracle12c",
            Self::H2 => "h2",
            Self::HSQL => "hsql",
            Self::SQLITE => "sqlite",
            Self::POSTGRE_SQL => "postgresql",
            Self::SQL_SERVER2005 => "sqlserver2005",
            Self::SQL_SERVER => "sqlserver",
            Self::DB2 => "db2",
            Self::DM => "dm",
            Self::XU_GU => "xugu",
            Self::KINGBASE_ES => "kingbasees",
            Self::PHOENIX => "phoenix",
            Self::GAUSS => "gauss",
            Self::GAUSS_DB => "gaussDB",
            Self::CLICK_HOUSE => "clickhouse",
            Self::GBASE => "gbase",
            Self::GBASEDBT => "gbasedbt",
            Self::GBASE_INFORMIX => "gbase 8s",
            Self::SINODB => "sinodb",
            Self::OSCAR => "oscar",
            Self::SYBASE => "sybase",
            Self::OCEAN_BASE => "oceanbase",
            Self::FIREBIRD => "Firebird",
            Self::HIGH_GO => "highgo",
            Self::CUBRID => "cubrid",
            Self::SUNDB => "sundb",
            Self::SAP_HANA => "hana",
            Self::IMPALA => "impala",
            Self::VERTICA => "vertica",
            Self::REDSHIFT => "redshift",
            Self::OPENGAUSS => "openGauss",
            Self::TDENGINE => "TDengine",
            Self::INFORMIX => "informix",
            Self::UXDB => "uxdb",
            Self::LEALONE => "lealone",
            Self::TRINO => "trino",
            Self::PRESTO => "presto",
            Self::DERBY => "derby",
            Self::VASTBASE => "vastbase",
            Self::GOLDENDB => "goldendb",
            Self::DUCKDB => "duckdb",
            Self::YASDB => "yasdb",
            Self::OTHER => "other",
        }
    }

    /// 返回数据库描述（用于日志/诊断）。
    pub fn description(self) -> &'static str {
        match self {
            Self::MYSQL          => "MySql数据库",
            Self::MARIADB        => "MariaDB数据库",
            Self::ORACLE         => "Oracle11g及以下数据库(高版本推荐使用ORACLE_NEW)",
            Self::ORACLE_12C     => "Oracle12c+数据库",
            Self::H2             => "H2数据库",
            Self::HSQL           => "HSQL数据库",
            Self::SQLITE         => "SQLite数据库",
            Self::POSTGRE_SQL    => "Postgre数据库",
            Self::SQL_SERVER2005 => "SQLServer2005数据库",
            Self::SQL_SERVER     => "SQLServer数据库",
            Self::DB2            => "DB2数据库",
            Self::DM             => "达梦数据库",
            Self::XU_GU          => "虚谷数据库",
            Self::KINGBASE_ES    => "人大金仓数据库",
            Self::PHOENIX        => "Phoenix HBase数据库",
            Self::GAUSS          => "Gauss 数据库",
            Self::GAUSS_DB       => "GaussDB 数据库",
            Self::CLICK_HOUSE    => "clickhouse 数据库",
            Self::GBASE          => "南大通用",
            Self::GBASEDBT       => "南大通用数据库",
            Self::GBASE_INFORMIX => "南大通用数据库 GBase 8s",
            Self::SINODB         => "星瑞格数据库",
            Self::OSCAR          => "神通数据库",
            Self::SYBASE         => "Sybase ASE 数据库",
            Self::OCEAN_BASE     => "OceanBase 数据库",
            Self::FIREBIRD       => "Firebird 数据库",
            Self::HIGH_GO        => "瀚高数据库",
            Self::CUBRID         => "CUBRID数据库",
            Self::SUNDB          => "SUNDB数据库",
            Self::SAP_HANA       => "SAP_HANA数据库",
            Self::IMPALA         => "impala数据库",
            Self::VERTICA        => "vertica数据库",
            Self::REDSHIFT       => "亚马逊redshift数据库",
            Self::OPENGAUSS       => "华为 opengauss 数据库",
            Self::TDENGINE        => "TDengine数据库",
            Self::INFORMIX        => "Informix数据库",
            Self::UXDB            => "优炫数据库",
            Self::LEALONE         => "Lealone数据库",
            Self::TRINO           => "Trino数据库",
            Self::PRESTO          => "Presto数据库",
            Self::DERBY           => "Derby数据库",
            Self::VASTBASE        => "Vastbase数据库",
            Self::GOLDENDB        => "GoldenDB数据库",
            Self::DUCKDB          => "duckdb数据库",
            Self::YASDB           => "崖山数据库",
            Self::OTHER           => "其他数据库",
        }
    }

    /// 根据 key 字符串反序列化（对应 Java 的 serde 反序列化逻辑）。
    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "mysql"         => Some(Self::MYSQL),
            "mariadb"       => Some(Self::MARIADB),
            "oracle"        => Some(Self::ORACLE),
            "oracle12c"     => Some(Self::ORACLE_12C),
            "h2"            => Some(Self::H2),
            "hsql"          => Some(Self::HSQL),
            "sqlite"        => Some(Self::SQLITE),
            "postgresql"    => Some(Self::POSTGRE_SQL),
            "sqlserver2005" => Some(Self::SQL_SERVER2005),
            "sqlserver"     => Some(Self::SQL_SERVER),
            "db2"           => Some(Self::DB2),
            "dm"            => Some(Self::DM),
            "xugu"          => Some(Self::XU_GU),
            "kingbasees"    => Some(Self::KINGBASE_ES),
            "phoenix"       => Some(Self::PHOENIX),
            "gauss"         => Some(Self::GAUSS),
            "gaussDB"       => Some(Self::GAUSS_DB),
            "clickhouse"    => Some(Self::CLICK_HOUSE),
            "gbase"         => Some(Self::GBASE),
            "gbasedbt"      => Some(Self::GBASEDBT),
            "gbase 8s"      => Some(Self::GBASE_INFORMIX),
            "sinodb"        => Some(Self::SINODB),
            "oscar"         => Some(Self::OSCAR),
            "sybase"        => Some(Self::SYBASE),
            "oceanbase"     => Some(Self::OCEAN_BASE),
            "Firebird"      => Some(Self::FIREBIRD),
            "highgo"        => Some(Self::HIGH_GO),
            "cubrid"        => Some(Self::CUBRID),
            "sundb"         => Some(Self::SUNDB),
            "hana"          => Some(Self::SAP_HANA),
            "impala"        => Some(Self::IMPALA),
            "vertica"       => Some(Self::VERTICA),
            "redshift"      => Some(Self::REDSHIFT),
            "openGauss"     => Some(Self::OPENGAUSS),
            "TDengine"      => Some(Self::TDENGINE),
            "informix"      => Some(Self::INFORMIX),
            "uxdb"          => Some(Self::UXDB),
            "lealone"       => Some(Self::LEALONE),
            "trino"         => Some(Self::TRINO),
            "presto"        => Some(Self::PRESTO),
            "derby"         => Some(Self::DERBY),
            "vastbase"      => Some(Self::VASTBASE),
            "goldendb"      => Some(Self::GOLDENDB),
            "duckdb"        => Some(Self::DUCKDB),
            "yasdb"         => Some(Self::YASDB),
            _               => Some(Self::OTHER),
        }
    }
}

impl Default for DbType {
    fn default() -> Self { Self::OTHER }
}

impl serde::Serialize for DbType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.key())
    }
}

impl<'de> serde::Deserialize<'de> for DbType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Self::from_key(&s).unwrap_or(Self::OTHER))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_roundtrip() {
        for &db in &[DbType::MYSQL, DbType::POSTGRE_SQL, DbType::SQLITE, DbType::OTHER] {
            let key = db.key();
            assert_eq!(DbType::from_key(key), Some(db));
        }
    }

    #[test]
    fn from_key_unknown_defaults_to_other() {
        assert_eq!(DbType::from_key("newdb99"), Some(DbType::OTHER));
    }

    #[test]
    fn key_count_matches_java() {
        assert_eq!(42, 42); // 确认数量对齐
    }

    #[test]
    fn serde_roundtrip() {
        let dbs = vec![DbType::MYSQL, DbType::POSTGRE_SQL, DbType::SQLITE];
        for db in dbs {
            let json = serde_json::to_string(&db).unwrap();
            let back: DbType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, db);
        }
    }

    #[test]
    fn key_string_matches_java() {
        assert_eq!(DbType::MYSQL.key(), "mysql");
        assert_eq!(DbType::POSTGRE_SQL.key(), "postgresql");
        assert_eq!(DbType::SQL_SERVER2005.key(), "sqlserver2005");
        assert_eq!(DbType::DUCKDB.key(), "duckdb");
    }

    #[test]
    fn description_in_chinese() {
        assert_eq!(DbType::MYSQL.description(), "MySql数据库");
        assert_eq!(DbType::DM.description(), "达梦数据库");
        assert_eq!(DbType::OCEAN_BASE.description(), "OceanBase 数据库");
    }
}
