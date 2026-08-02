//! 国际化上下文（对标 Java `I18nContext` 的 ThreadLocal Locale）。
//!
//! 使用 `tokio::task::LocalKey` 或 `std::thread::LocalKey` 管理当前请求的语言环境。
//! 在 Web 应用中，每个请求的 Locale 通常由中间件设置，请求结束后自动清除。

use std::cell::RefCell;

/// 默认语言环境（当上下文未设置 Locale 时使用）。
const DEFAULT_LOCALE: &str = "zh_CN";

thread_local! {
    /// 当前线程的语言环境（对标 Java `TransmittableThreadLocal<Locale>`）。
    ///
    /// 使用 `RefCell<Option<String>>` 实现线程局部可变状态。
    /// 请求中间件在请求开始时设置，请求结束时清除。
    static CURRENT_LOCALE: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// 国际化上下文管理器。
///
/// 对应 Java：MyBatis-Plus-Enhance 的 `I18nContext`。
/// 提供线程安全的 Locale 读写接口，支持 `set` / `get` / `clear` 操作。
///
/// # 使用示例
///
/// ```ignore
/// use rbatis_plus_extension::i18n::context::I18nContext;
///
/// // 在请求中间件中设置 Locale
/// I18nContext::set("en_US");
///
/// // 在拦截器中获取当前 Locale
/// let locale = I18nContext::get();
/// assert_eq!(locale, "en_US");
///
/// // 请求结束后清除
/// I18nContext::clear();
/// ```
pub struct I18nContext;

impl I18nContext {
    /// 设置当前线程的语言环境。
    ///
    /// 对应 Java：`I18nContext.set(Locale)`
    pub fn set(locale: impl Into<String>) {
        CURRENT_LOCALE.with(|cell| {
            *cell.borrow_mut() = Some(locale.into());
        });
    }

    /// 获取当前线程的语言环境。
    ///
    /// 若未设置，返回默认语言环境 `"zh_CN"`。
    ///
    /// 对应 Java：`I18nContext.get()`
    pub fn get() -> String {
        CURRENT_LOCALE.with(|cell| {
            cell.borrow()
                .clone()
                .unwrap_or_else(|| DEFAULT_LOCALE.to_string())
        })
    }

    /// 获取当前线程的语言环境（若未设置返回 `None`）。
    pub fn try_get() -> Option<String> {
        CURRENT_LOCALE.with(|cell| cell.borrow().clone())
    }

    /// 清除当前线程的语言环境。
    ///
    /// 对应 Java：`I18nContext.remove()`
    pub fn clear() {
        CURRENT_LOCALE.with(|cell| {
            *cell.borrow_mut() = None;
        });
    }

    /// 获取默认语言环境。
    pub fn default_locale() -> &'static str {
        DEFAULT_LOCALE
    }

    /// 在指定语言环境下执行闭包。
    ///
    /// 执行完毕后自动恢复之前的语言环境（RAII 风格）。
    ///
    /// # 使用示例
    ///
    /// ```ignore
    /// use rbatis_plus_extension::i18n::context::I18nContext;
    ///
    /// I18nContext::set("zh_CN");
    /// let result = I18nContext::with_locale("en_US", || {
    ///     // 此处 Locale 为 "en_US"
    ///     I18nContext::get()
    /// });
    /// // 此处 Locale 恢复为 "zh_CN"
    /// assert_eq!(result, "en_US");
    /// ```
    pub fn with_locale<F, R>(locale: impl Into<String>, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let previous = Self::try_get();
        Self::set(locale);
        let result = f();
        match previous {
            Some(prev) => Self::set(prev),
            None => Self::clear(),
        }
        result
    }
}

impl std::fmt::Debug for I18nContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("I18nContext")
            .field("current_locale", &Self::try_get())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_set_and_get() {
        I18nContext::clear();
        I18nContext::set("en_US");
        assert_eq!(I18nContext::get(), "en_US");
        I18nContext::clear();
    }

    #[test]
    fn test_context_default_locale() {
        I18nContext::clear();
        assert_eq!(I18nContext::get(), "zh_CN");
    }

    #[test]
    fn test_context_try_get_none_when_unset() {
        I18nContext::clear();
        assert!(I18nContext::try_get().is_none());
    }

    #[test]
    fn test_context_clear() {
        I18nContext::set("ja_JP");
        assert_eq!(I18nContext::get(), "ja_JP");
        I18nContext::clear();
        assert!(I18nContext::try_get().is_none());
    }

    #[test]
    fn test_context_with_locale_restores_previous() {
        I18nContext::set("zh_CN");
        let inner = I18nContext::with_locale("en_US", || {
            assert_eq!(I18nContext::get(), "en_US");
            "done"
        });
        assert_eq!(inner, "done");
        assert_eq!(I18nContext::get(), "zh_CN");
        I18nContext::clear();
    }

    #[test]
    fn test_context_with_locale_restores_none() {
        I18nContext::clear();
        I18nContext::with_locale("ko_KR", || {
            assert_eq!(I18nContext::get(), "ko_KR");
        });
        assert!(I18nContext::try_get().is_none());
    }
}
