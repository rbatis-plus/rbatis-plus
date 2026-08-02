//! `MybatisPlusEnhanceInterceptor` 桥接到 rbatis 拦截链的集成测试。
//!
//! 验证：
//! 1. `install()` 后，InnerInterceptor 的 before/after 钩子在真实
//!    rbatis 执行链（`RBatis::exec` / `RBatis::query`）上生效；
//! 2. 事务事件（commit/rollback）转发到 `on_transaction_event`；
//! 3. （`cache` feature）SQL 改写拦截器先于缓存执行——缓存键使用
//!    改写后的 SQL，不同改写结果不会串缓存。

#![allow(mismatched_lifetime_syntaxes)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use futures::future::BoxFuture;
use futures::Stream;
use rbatis::executor::Executor;
use rbatis::intercept::Action;
use rbatis::plugin::transaction::{
    TransactionEvent, TransactionEventType,
};
use rbatis::rbdc::db::{ConnectOptions, Connection, Driver, ExecResult, MetaData, Row};
use rbatis::rbdc::rt::block_on;
use rbatis::{Error, RBatis};
use rbs::Value;
use std::pin::Pin;

use rbatis_plus_extension::inner::{
    BlockAttackInnerInterceptor, InnerInterceptor, MybatisPlusEnhanceInterceptor,
};

// ---------------------------------------------------------------------------
// Mock driver that counts database calls
// ---------------------------------------------------------------------------

static QUERY_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone)]
struct CountingMockDriver;

impl Driver for CountingMockDriver {
    fn name(&self) -> &str {
        "enhance-bridge-test"
    }
    fn connect(
        &self,
        _url: &str,
    ) -> BoxFuture<'_, Result<Box<dyn Connection>, rbatis::rbdc::Error>> {
        Box::pin(async { Ok(Box::new(CountingConn) as Box<dyn Connection>) })
    }
    fn connect_opt<'a>(
        &'a self,
        _opt: &'a dyn ConnectOptions,
    ) -> BoxFuture<'a, Result<Box<dyn Connection>, rbatis::rbdc::Error>> {
        Box::pin(async { Ok(Box::new(CountingConn) as Box<dyn Connection>) })
    }
    fn default_option(&self) -> Box<dyn ConnectOptions> {
        Box::new(MockOpts)
    }
}

#[derive(Clone, Debug)]
struct CountingMeta;
impl MetaData for CountingMeta {
    fn column_len(&self) -> usize {
        1
    }
    fn column_name(&self, _i: usize) -> String {
        "v".into()
    }
    fn column_type(&self, _i: usize) -> String {
        "I64".into()
    }
}

#[derive(Clone, Debug)]
struct CountingRow;
impl Row for CountingRow {
    fn meta_data(&self) -> Box<dyn MetaData> {
        Box::new(CountingMeta)
    }
    fn get(&mut self, _i: usize) -> Result<Value, rbatis::rbdc::Error> {
        Ok(Value::I64(1))
    }
}

#[derive(Clone, Debug)]
struct CountingConn;
impl Connection for CountingConn {
    fn exec_rows(
        &mut self,
        _sql: &str,
        _p: Vec<Value>,
    ) -> BoxFuture<
        '_,
        Result<
            Pin<Box<dyn Stream<Item = Result<Box<dyn Row>, rbatis::rbdc::Error>> + Send + '_>>,
            rbatis::rbdc::Error,
        >,
    > {
        QUERY_COUNT.fetch_add(1, Ordering::SeqCst);
        Box::pin(async {
            let row = Box::new(CountingRow) as Box<dyn Row>;
            let s: Pin<Box<dyn Stream<Item = Result<Box<dyn Row>, rbatis::rbdc::Error>> + Send + '_>> =
                Box::pin(futures::stream::iter(vec![Ok(row)]));
            Ok(s)
        })
    }
    fn exec(
        &mut self,
        _sql: &str,
        _p: Vec<Value>,
    ) -> BoxFuture<'_, Result<ExecResult, rbatis::rbdc::Error>> {
        Box::pin(async {
            Ok(ExecResult {
                rows_affected: 1,
                last_insert_id: Value::Null,
            })
        })
    }
    fn close(&mut self) -> BoxFuture<'_, Result<(), rbatis::rbdc::Error>> {
        Box::pin(async { Ok(()) })
    }
    fn ping(&mut self) -> BoxFuture<'_, Result<(), rbatis::rbdc::Error>> {
        Box::pin(async { Ok(()) })
    }
}

#[derive(Clone, Debug)]
struct MockOpts;
impl ConnectOptions for MockOpts {
    fn connect(&self) -> BoxFuture<'_, Result<Box<dyn Connection>, rbatis::rbdc::Error>> {
        Box::pin(async { Ok(Box::new(CountingConn) as Box<dyn Connection>) })
    }
    fn set_uri(&mut self, _u: &str) -> Result<(), rbatis::rbdc::Error> {
        Ok(())
    }
}

fn setup_rb() -> RBatis {
    QUERY_COUNT.store(0, Ordering::SeqCst);
    let rb = RBatis::new();
    rb.init(CountingMockDriver, "mock://test").unwrap();
    rb
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn block_attack_blocks_full_table_update_via_bridge() {
    let rb = setup_rb();
    let mut enhance = MybatisPlusEnhanceInterceptor::new();
    enhance.add_inner_interceptor(BlockAttackInnerInterceptor::new());
    Arc::new(enhance).install(&rb);

    block_on(async move {
        // 无 WHERE 的 UPDATE 必须被 BlockAttack 拦截（经桥接）。
        let err = rb.exec("update t set x = 1", vec![]).await.unwrap_err();
        assert!(
            err.to_string().contains("Prohibition of full table update"),
            "block attack must fire via bridge: {err}"
        );
    });
}

#[test]
fn block_attack_allows_where_clause() {
    let rb = setup_rb();
    let mut enhance = MybatisPlusEnhanceInterceptor::new();
    enhance.add_inner_interceptor(BlockAttackInnerInterceptor::new());
    Arc::new(enhance).install(&rb);

    block_on(async move {
        // 带 WHERE 的 UPDATE 正常放行。
        let r = rb.exec("update t set x = 1 where id = ?", vec![Value::I32(1)]).await;
        assert!(r.is_ok(), "where-clause update must pass: {:?}", r.err());
    });
}

/// 记录事务事件的 InnerInterceptor（事件存储与测试共享）。
#[derive(Debug, Clone)]
struct EventRecorder {
    events: Arc<Mutex<Vec<TransactionEventType>>>,
}

impl EventRecorder {
    fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl InnerInterceptor for EventRecorder {
    async fn on_transaction_event(&self, event: &TransactionEvent) {
        self.events.lock().unwrap().push(event.event_type);
    }
}

#[test]
fn transaction_events_forwarded_to_inner() {
    let rb = setup_rb();
    let recorder = EventRecorder::new();
    let events = Arc::clone(&recorder.events);
    let mut enhance = MybatisPlusEnhanceInterceptor::new();
    enhance.add_inner_interceptor(recorder);
    Arc::new(enhance).install(&rb);

    block_on(async move {
        let tx = rb.acquire_begin().await.unwrap();
        tx.commit().await.unwrap();
    });
    let events = events.lock().unwrap();
    assert!(
        events.contains(&TransactionEventType::CommitSuccess),
        "inner interceptor must receive commit event: {events:?}"
    );
}

// ---------------------------------------------------------------------------
// Ordering: SQL-rewriting interceptor runs BEFORE cache (cache feature)
// ---------------------------------------------------------------------------

/// 模拟分页/租户的 SQL 改写拦截器：给 SELECT 追加 `limit <n>`。
#[derive(Debug)]
struct LimitRewriter {
    limit: u64,
}

#[async_trait]
impl InnerInterceptor for LimitRewriter {
    async fn before_query(
        &self,
        _executor: &dyn Executor,
        sql: &mut String,
        _args: &mut Vec<Value>,
        _result: &mut Result<Value, Error>,
    ) -> Result<Action, Error> {
        if !sql.contains("limit") {
            sql.push_str(&format!(" limit {}", self.limit));
        }
        Ok(Action::Next)
    }
}

#[cfg(feature = "cache")]
#[test]
fn cache_sees_rewritten_sql_when_enhance_runs_first() {
    use rbatis_cache::{CachePolicy, LocalBackend, RbatisCacheExt, RbatisCacheInterceptor};

    let rb = setup_rb();
    // 先装缓存，再装增强（后安装者位于链首：增强改写先于缓存判定）。
    let cache = RbatisCacheInterceptor::new(
        "order_ns",
        Arc::new(LocalBackend::new()),
        CachePolicy::default(),
    );
    let listener = cache.listener();
    rb.install_cache(Arc::new(cache), Some(Arc::new(listener)));

    let mut enhance = MybatisPlusEnhanceInterceptor::new();
    enhance.add_inner_interceptor(LimitRewriter { limit: 10 });
    Arc::new(enhance).install(&rb);

    block_on(async move {
        // 同一 SQL 经改写后（limit 10）查询两次：第一次 miss 走 DB，第二次命中缓存。
        let _ = rb.query("select * from t", vec![]).await.unwrap();
        assert_eq!(QUERY_COUNT.load(Ordering::SeqCst), 1);
        let _ = rb.query("select * from t", vec![]).await.unwrap();
        assert_eq!(
            QUERY_COUNT.load(Ordering::SeqCst),
            1,
            "rewritten SQL must be cached consistently (rewriter before cache)"
        );
    });
}
