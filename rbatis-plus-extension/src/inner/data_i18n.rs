//! 结果国际化增强拦截器（对标 Java `DataI18nInnerInterceptor`）。
//!
//! 在 SELECT 后对结果集中的国际化字段进行翻译。
//! 阶段：`RESULT_I18N`（500），在解密之后、观测之前执行。
//!
//! # 对应 Java
//!
//! - `com.baomidou.mybatisplus.enhance.plugins.inner.DataI18nInnerInterceptor`
//!   （mybatis-plus-enhance-extension/.../plugins/inner/DataI18nInnerInterceptor.java，114 行）

use async_trait::async_trait;
use rbatis::executor::Executor;
use rbatis::Error;
use rbs::Value;
use std::sync::Arc;

use crate::i18n::context::I18nContext;
use crate::i18n::handler::DataI18nHandler;
use crate::inner::enhance_interceptor::EnhanceInnerInterceptor;
use crate::inner::enhance_phase::EnhancePhase;
use crate::inner::inner_interceptor::InnerInterceptor;

/// 结果国际化增强拦截器（对标 Java `DataI18nInnerInterceptor`）。
///
/// 职责：
/// - `after_query`：从 `I18nContext` 获取当前 Locale，调用
///   `DataI18nHandler::handle` 翻译结果集
///
/// # 对应 Java
///
/// - `com.baomidou.mybatisplus.enhance.plugins.inner.DataI18nInnerInterceptor`
///
/// # 使用示例
///
/// ```ignore
/// use rbatis_plus_extension::inner::DataI18nInnerInterceptor;
/// use rbatis_plus_extension::i18n::DefaultDataI18nHandler;
///
/// let handler = DefaultDataI18nHandler::new()
///     .with_i18n_column("_i18n")
///     .with_target_columns(&["name", "desc"]);
///
/// let interceptor = DataI18nInnerInterceptor::new(Box::new(handler));
/// ```
pub struct DataI18nInnerInterceptor {
    /// 国际化处理器。
    handler: Arc<dyn DataI18nHandler>,
}

impl DataI18nInnerInterceptor {
    /// 创建结果国际化拦截器。
    ///
    /// 对应 Java：`DataI18nInnerInterceptor(DataI18nHandler handler)`
    pub fn new(handler: Box<dyn DataI18nHandler>) -> Self {
        Self {
            handler: Arc::from(handler),
        }
    }

    /// 获取处理器引用。
    pub fn handler(&self) -> &dyn DataI18nHandler {
        self.handler.as_ref()
    }
}

impl Clone for DataI18nInnerInterceptor {
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
        }
    }
}

impl std::fmt::Debug for DataI18nInnerInterceptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataI18nInnerInterceptor").finish()
    }
}

#[async_trait]
impl InnerInterceptor for DataI18nInnerInterceptor {
    /// SELECT 后对结果集进行国际化翻译。
    ///
    /// 对应 Java：`DataI18nInnerInterceptor.afterQuery(...)`
    ///
    /// 流程：
    /// 1. 从 `I18nContext` 获取当前线程的 Locale
    /// 2. 若 Locale 未设置，使用默认值 "zh_CN"
    /// 3. 调用 `handler.handle(locale, ms_id, results)` 执行翻译
    async fn after_query(
        &self,
        _executor: &dyn Executor,
        _sql: &str,
        result: &mut Result<Value, Error>,
    ) -> Result<(), Error> {
        if let Ok(value) = result {
            // 获取当前语言环境（对标 Java TTL ThreadLocal）
            let locale = I18nContext::get();

            // 调用处理器翻译结果集
            self.handler.handle(&locale, "", value)?;
        }
        Ok(())
    }
}

#[async_trait]
impl EnhanceInnerInterceptor for DataI18nInnerInterceptor {
    /// 声明阶段为结果国际化（500）。
    ///
    /// 对应 Java：`DataI18nInnerInterceptor.phase()`
    fn phase(&self) -> EnhancePhase {
        EnhancePhase::RESULT_I18N
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::handler::DefaultDataI18nHandler;
    use crate::i18n::handler::NoopDataI18nHandler;
    use crate::i18n::context::I18nContext;
    use crate::i18n::KeyValuePair;
    use rbs::value::map::ValueMap;

    /// 创建带翻译数据的测试拦截器。
    fn create_test_interceptor() -> DataI18nInnerInterceptor {
        let mut handler = DefaultDataI18nHandler::new()
            .with_i18n_column("_i18n")
            .with_target_column("name");

        handler.add_resource_bundle(
            "zh_CN",
            vec![
                KeyValuePair::new("product.phone.name", "手机"),
                KeyValuePair::new("product.laptop.name", "笔记本电脑"),
            ],
        );

        handler.add_resource_bundle(
            "en_US",
            vec![
                KeyValuePair::new("product.phone.name", "Phone"),
                KeyValuePair::new("product.laptop.name", "Laptop"),
            ],
        );

        DataI18nInnerInterceptor::new(Box::new(handler))
    }

    #[test]
    fn test_phase_is_result_i18n() {
        let interceptor = create_test_interceptor();
        assert_eq!(interceptor.phase(), EnhancePhase::RESULT_I18N);
    }

    #[test]
    fn test_phase_order_after_decryption() {
        // RESULT_I18N (500) 应在 RESULT_DECRYPTION (400) 之后
        let interceptor = create_test_interceptor();
        assert!(interceptor.phase().order() > EnhancePhase::RESULT_DECRYPTION.order());
        assert!(interceptor.phase().order() < EnhancePhase::OBSERVATION.order());
    }

    #[test]
    fn test_debug_impl() {
        let interceptor = create_test_interceptor();
        let debug_str = format!("{:?}", interceptor);
        assert!(debug_str.contains("DataI18nInnerInterceptor"));
    }

    #[test]
    fn test_clone_impl() {
        let interceptor = create_test_interceptor();
        let cloned = interceptor.clone();
        assert_eq!(cloned.phase(), EnhancePhase::RESULT_I18N);
    }

    #[test]
    fn test_noop_handler_does_not_modify() {
        let interceptor = DataI18nInnerInterceptor::new(Box::new(NoopDataI18nHandler));

        let mut map = ValueMap::new();
        map.insert(Value::String("_i18n".into()), Value::String("test".into()));
        map.insert(Value::String("name".into()), Value::String("original".into()));
        let mut value = Value::Map(map);

        let result = interceptor.handler.handle("zh_CN", "", &mut value);
        assert!(result.is_ok());

        if let Value::Map(ref m) = value {
            let name = m.get(&Value::String("name".into()));
            if let Value::String(s) = name {
                assert_eq!(s, "original", "NoopHandler 不应修改值");
            }
        }
    }

    #[test]
    fn test_handler_translates_with_locale() {
        let mut handler = DefaultDataI18nHandler::new()
            .with_i18n_column("_i18n")
            .with_target_column("name");

        handler.add_resource_bundle(
            "zh_CN",
            vec![KeyValuePair::new("item.name", "物品")],
        );

        let interceptor = DataI18nInnerInterceptor::new(Box::new(handler));

        let mut map = ValueMap::new();
        map.insert(Value::String("_i18n".into()), Value::String("item".into()));
        map.insert(Value::String("name".into()), Value::String("Item".into()));
        let mut value = Value::Map(map);

        I18nContext::set("zh_CN");
        let result = interceptor.handler.handle(&I18nContext::get(), "", &mut value);
        assert!(result.is_ok());

        if let Value::Map(ref m) = value {
            let name = m.get(&Value::String("name".into()));
            if let Value::String(s) = name {
                assert_eq!(s, "物品", "应根据 locale 翻译");
            }
        }
        I18nContext::clear();
    }
}
