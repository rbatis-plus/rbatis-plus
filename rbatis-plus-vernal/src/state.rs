//! 应用状态管理（对标 mybatis-plus-spring `SqlSessionFactory` 注册）。
//!
//! 提供 RBatis 实例的共享状态管理，用于 axum/actix-web 的 State 注入。

/// 应用状态（对标 Java Spring 的 `@Bean RBatis`）。
///
/// 包含 RBatis 实例和配置，用于 axum 的 `State` 注入。
///
/// # Example
///
/// ```ignore
/// use rbatis_plus_vernal::AppState;
/// use rbatis_plus_vernal::VernalConfig;
///
/// // 方式 1：使用已初始化的 RBatis
/// let rb = rbatis::RBatis::new();
/// // rb.init(MySqlDriver{}, "mysql://root:123456@localhost:3306/test")?;
/// let config = VernalConfig::builder()
///     .url("mysql://root:123456@localhost:3306/test")
///     .build();
/// let state = AppState::with_rbatis(rb, config);
///
/// // 方式 2：直接传入连接池
/// let state = AppState::new(rb, config);
/// // 在 axum 中使用: .with_state(state)
/// ```
pub struct AppState {
    /// RBatis 实例。
    pub rb: rbatis::RBatis,
    /// Vernal 配置。
    pub config: super::VernalConfig,
}

impl AppState {
    /// 使用已初始化的 RBatis 实例创建状态。
    ///
    /// 对应 Java `MybatisPlusAutoConfiguration` 中的 `SqlSessionFactory` 注册。
    pub fn new(rb: rbatis::RBatis, config: super::VernalConfig) -> Self {
        Self { rb, config }
    }

    /// 获取 RBatis 实例的引用。
    pub fn rb(&self) -> &rbatis::RBatis {
        &self.rb
    }

    /// 获取配置的引用。
    pub fn config(&self) -> &super::VernalConfig {
        &self.config
    }
}

// RBatis 内部是 Arc 包装，Clone 是安全的
impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            rb: self.rb.clone(),
            config: self.config.clone(),
        }
    }
}
