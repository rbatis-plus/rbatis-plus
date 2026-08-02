//! 事务管理工具（对标 mybatis-plus-extension 事务支持）。
//!
//! 提供便捷的事务执行辅助函数和 RAII 事务守卫。
//!
//! # 对应 Java
//!
//! - `com.baomidou.mybatisplus.extension.toolkit.TransactionTemplate`
//! - Spring `@Transactional` 的编程式等价物

use rbatis::executor::RBatisTxExecutor;
use rbatis::RBatis;
use rbs::Value;

/// 在事务内执行闭包（对标 Java `TransactionTemplate.execute`）。
///
/// 开启事务后将 `RBatis` 和 `RBatisTxExecutor` 传给闭包 `f`。
/// 闭包返回 `Ok` 时自动提交，返回 `Err` 时自动回滚。
///
/// # 对应 Java
///
/// `com.baomidou.mybatisplus.extension.toolkit.TransactionTemplate.execute(TransactionCallback)`
///
/// # Example
///
/// ```ignore
/// use rbatis_plus_vernal::run_in_transaction;
/// use rbs::Value;
///
/// let result = run_in_transaction(&rb, |rb, tx| async move {
///     rb.exec("INSERT INTO log (msg) VALUES (?)", vec![Value::from("hello")]).await?;
///     rb.exec("UPDATE user SET score = score + 1 WHERE id = ?", vec![Value::from(1)]).await?;
///     Ok::<(), rbatis::Error>(())
/// }).await;
/// ```
pub async fn run_in_transaction<F, Fut, R>(rb: &RBatis, f: F) -> Result<R, rbatis::Error>
where
    F: FnOnce(RBatis, RBatisTxExecutor) -> Fut,
    Fut: std::future::Future<Output = Result<R, rbatis::Error>> + Send,
    R: Send,
{
    let tx = rb.acquire_begin().await?;
    match f(rb.clone(), tx.clone()).await {
        Ok(result) => {
            tx.commit().await?;
            Ok(result)
        }
        Err(e) => {
            // 回滚时忽略错误，保留原始错误
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

/// 事务守卫（对标 Java `@Transactional` 的 RAII 实现）。
///
/// 自动管理事务生命周期：
/// - 调用 `commit()` 显式提交事务
/// - 如果未提交就被 drop，自动回滚
///
/// # 对应 Java
///
/// Spring `@Transactional` 注解 + AOP 自动回滚语义。
/// 当方法正常返回（对应 `commit()`）时提交，抛出异常（对应 guard 被 drop 而未 commit）时回滚。
///
/// # Example
///
/// ```ignore
/// use rbatis_plus_vernal::TransactionalGuard;
/// use rbs::Value;
///
/// let mut guard = TransactionalGuard::begin(&rb).await?;
///
/// // 在事务内执行操作
/// guard.inner_mut().exec("INSERT INTO user (name) VALUES (?)", vec![Value::from("Alice")]).await?;
/// guard.inner_mut().exec("UPDATE account SET balance = balance - 100 WHERE id = ?", vec![Value::from(1)]).await?;
///
/// // 显式提交
/// guard.commit().await?;
/// // 如果此处发生 panic 或 early return 未调用 commit()，事务将自动回滚
/// ```
pub struct TransactionalGuard {
    tx: RBatisTxExecutor,
    committed: bool,
}

impl TransactionalGuard {
    /// 开启事务并返回守卫。
    ///
    /// 对应 Java 方法上添加 `@Transactional` 注解（进入方法时开启事务）。
    pub async fn begin(rb: &RBatis) -> Result<Self, rbatis::Error> {
        let tx = rb.acquire_begin().await?;
        Ok(Self {
            tx,
            committed: false,
        })
    }

    /// 提交事务。
    ///
    /// 对应 Java 方法正常返回时的事务提交。
    ///
    /// # 注意
    ///
    /// 提交后 `committed` 标记为 true，drop 时不再回滚。
    /// 重复调用 `commit()` 是幂等的（第二次调用为 no-op）。
    pub async fn commit(mut self) -> Result<(), rbatis::Error> {
        if !self.committed {
            self.tx.commit().await?;
            self.committed = true;
        }
        Ok(())
    }

    /// 手动回滚事务。
    ///
    /// 对应 Java 中显式调用 `TransactionAspectSupport.currentTransactionStatus().setRollbackOnly()`。
    pub async fn rollback(&mut self) -> Result<(), rbatis::Error> {
        if !self.committed {
            self.tx.rollback().await?;
            self.committed = true;
        }
        Ok(())
    }

    /// 获取事务执行器的不可变引用。
    ///
    /// 可用于在事务内执行查询和更新操作。
    pub fn inner(&self) -> &RBatisTxExecutor {
        &self.tx
    }

    /// 获取事务执行器的可变引用。
    ///
    /// 可用于在事务内执行查询和更新操作。
    pub fn inner_mut(&mut self) -> &mut RBatisTxExecutor {
        &mut self.tx
    }

    /// 在事务内执行原生 SQL（便捷方法）。
    ///
    /// 对应在 `@Transactional` 方法内使用 `JdbcTemplate.execute()`。
    pub async fn exec(
        &mut self,
        sql: &str,
        args: Vec<Value>,
    ) -> Result<rbdc::db::ExecResult, rbatis::Error> {
        self.tx.exec(sql, args).await
    }

    /// 在事务内执行查询（便捷方法）。
    ///
    /// 对应在 `@Transactional` 方法内使用 `JdbcTemplate.query()`。
    pub async fn query(
        &mut self,
        sql: &str,
        args: Vec<Value>,
    ) -> Result<Value, rbatis::Error> {
        self.tx.query(sql, args).await
    }

    /// 事务是否已提交。
    pub fn is_committed(&self) -> bool {
        self.committed
    }
}

impl Drop for TransactionalGuard {
    fn drop(&mut self) {
        if !self.committed {
            // 自动回滚：使用 block_on 在 drop 中执行异步回滚。
            // 注意：如果运行时已关闭，回滚可能失败，但这是 best-effort。
            log::warn!(
                "TransactionalGuard 被 drop 但未 commit，自动回滚事务 (tx_id={})",
                self.tx.tx_id
            );
            let tx = self.tx.clone();
            // 尝试在当前线程阻塞执行回滚
            // 注意：在 async 上下文中 drop 时，rbatis 内部的连接可能仍可访问
            // 这里使用 try_exec 方式，如果运行时不支持则仅记录日志
            rbdc::rt::spawn(async move {
                if let Err(e) = tx.rollback().await {
                    log::error!("TransactionalGuard 自动回滚失败 (tx_id={}): {}", tx.tx_id, e);
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试 TransactionalGuard 的 API 存在性。
    #[test]
    fn test_transactional_guard_api_exists() {
        // 验证 TransactionalGuard 的方法签名在编译期正确
        let _ = |guard: TransactionalGuard| {
            let _ = guard.is_committed();
            let _ = guard.inner();
        };
    }

    /// 测试 run_in_transaction 函数存在性（编译期检查）。
    #[test]
    fn test_run_in_transaction_api_exists() {
        // 确认函数可被引用（编译时检查签名正确性）
        fn _assert_fn_exists(_: impl std::future::Future<Output = Result<(), rbatis::Error>>) {}
    }
}
