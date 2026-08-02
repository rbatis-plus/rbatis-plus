/// 主键 ID 类型枚举。
///
/// 对应 Java：`com.baomidou.mybatisplus.annotation.IdType`（mybatis-plus-annotation）
/// 文件来源参考：`mybatis-plus-annotation/src/main/java/com/baomidou/mybatisplus/annotation/IdType.java`
///
/// ```rust
/// use rbatis_plus_core::IdType;
///
/// assert_eq!(IdType::Auto.key(), 0);
/// assert_eq!(IdType::None.key(), 1);
/// assert_eq!(IdType::Input.key(), 2);
/// assert_eq!(IdType::AssignId.key(), 3);
/// assert_eq!(IdType::AssignUuid.key(), 4);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdType {
    /// 数据库 ID 自增（该策略保存前手动设置 ID 无效且会被自动生成的 ID 回写，
    /// 正确用法是保存对象插入成功后拿到自动生成的 ID，然后再去关联业务使用）。
    /// 该类型请确保数据库设置了 ID 自增，否则无效。
    ///
    /// 对应 Java：`IdType.AUTO(0)`
    Auto,
    /// 该类型为未设置主键类型（注解里等于跟随全局，全局里约等于 INPUT）。
    ///
    /// 对应 Java：`IdType.NONE(1)`
    None,
    /// 用户输入 ID。该类型可以通过自己注册自动填充插件进行填充。
    ///
    /// 对应 Java：`IdType.INPUT(2)`
    Input,
    /// 分配 ID（主键类型为 number 或 string），默认实现类为雪花算法生成器。
    ///
    /// 对应 Java：`IdType.ASSIGN_ID(3)`
    AssignId,
    /// 分配 UUID（主键类型为 string），默认实现类生成 UUID 并去除 "-"。
    ///
    /// 对应 Java：`IdType.ASSIGN_UUID(4)`
    AssignUuid,
}

impl IdType {
    /// 返回枚举对应的 key 整数值（用于序列化/反序列化、与 Java 配置文件兼容）。
    ///
    /// 对应 Java：`IdType.key` 字段（Lombok `@Getter` 生成 `getKey()`）。
    pub fn key(self) -> u32 {
        match self {
            Self::Auto       => 0,
            Self::None       => 1,
            Self::Input      => 2,
            Self::AssignId   => 3,
            Self::AssignUuid => 4,
        }
    }

    /// 根据整数值解析 IdType（反序列化用）。
    ///
    /// 未知值回退到 `None`，与 Java 未设置行为一致。
    pub fn from_key(key: u32) -> Self {
        match key {
            0 => Self::Auto,
            1 => Self::None,
            2 => Self::Input,
            3 => Self::AssignId,
            4 => Self::AssignUuid,
            _ => Self::None,
        }
    }
}

impl Default for IdType {
    fn default() -> Self { Self::None }
}

impl std::fmt::Display for IdType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto       => write!(f, "AUTO"),
            Self::None       => write!(f, "NONE"),
            Self::Input      => write!(f, "INPUT"),
            Self::AssignId   => write!(f, "ASSIGN_ID"),
            Self::AssignUuid => write!(f, "ASSIGN_UUID"),
        }
    }
}

impl serde::Serialize for IdType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(self.key())
    }
}

impl<'de> serde::Deserialize<'de> for IdType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let key = u32::deserialize(deserializer)?;
        Ok(Self::from_key(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_roundtrip() {
        for &id_type in &[IdType::Auto, IdType::None, IdType::Input, IdType::AssignId, IdType::AssignUuid] {
            assert_eq!(IdType::from_key(id_type.key()), id_type);
        }
    }

    #[test]
    fn from_key_unknown_defaults_to_none() {
        assert_eq!(IdType::from_key(99), IdType::None);
    }

    #[test]
    fn display_matches_java_string() {
        assert_eq!(IdType::Auto.to_string(), "AUTO");
        assert_eq!(IdType::None.to_string(), "NONE");
        assert_eq!(IdType::AssignId.to_string(), "ASSIGN_ID");
        assert_eq!(IdType::AssignUuid.to_string(), "ASSIGN_UUID");
    }

    #[test]
    fn serde_roundtrip() {
        let values = vec![IdType::Auto, IdType::None, IdType::Input, IdType::AssignId, IdType::AssignUuid];
        for v in values {
            let json = serde_json::to_string(&v).unwrap();
            let back: IdType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, v);
        }
    }

    #[test]
    fn java_compat_key_values() {
        // 与 Java 3.5.17 的 IdType.key() 保持一致
        assert_eq!(IdType::Auto.key(), 0);
        assert_eq!(IdType::None.key(), 1);
        assert_eq!(IdType::Input.key(), 2);
        assert_eq!(IdType::AssignId.key(), 3);
        assert_eq!(IdType::AssignUuid.key(), 4);
    }
}
