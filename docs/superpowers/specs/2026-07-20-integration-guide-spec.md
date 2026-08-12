# RBatis-Plus 集成指南规范

- **日期**：2026-07-20
- **状态**：已实施（部分进行中）
- **上游基线**：RBatis `master` @ `4050edd3dad03a113b8bb4f5818a006f11f2da78`

---

## 1. 集成目标

本文定义如何在不静默改变数据库语义的前提下引入 RBatis-Plus 缓存和拦截器能力。

### 1.1 Crate 拆分

```text
rbatis-plus              facade and re-exports (optional features select backends)
rbatis-plus-core         policy, SPI, interceptor, and shared protocols
rbatis-plus-extension    inner interceptors + enhance capabilities
rbatis-plus-macros       declarative annotations (proc-macro)
rbatis-plus-generator    code generator + template engines
rbatis-plus-sqlparser    SQL parser + dialect
rbatis-plus-vernal       axum/actix integration + SqlRunner + Transactions
```

---

## 2. Feature flags

| Feature | Crate | 说明 |
|---|---|---|
| `default` | rbatis-plus | 无额外依赖 |
| `cache` | rbatis-plus | 启用缓存子系统（rbatis-cache） |
| `axum` | rbatis-plus | 启用 Axum 集成 |

---

## 3. 最小集成示例

### 3.1 基础使用（无缓存）

```rust
use rbatis_plus::prelude::*;

#[derive(TableName, TableId, TableField)]
struct User {
    #[table_id]
    id: Option<i64>,
    name: Option<String>,
    age: Option<i32>,
}

#[py_sql("select * from user where id = #{id}")]
async fn select_user(rb: &RBatis, id: i64) -> Result<Option<User>, rbatis::Error> {
    impled!()
}
```

### 3.2 启用缓存

```rust
use rbatis_plus::prelude::*;
use rbatis_plus::CacheIntercept;
use std::sync::Arc;

// 1. 创建缓存后端
let store = Arc::new(MemoryCacheStore::builder()
    .max_capacity(50_000)
    .build()
    .await?);

// 2. 创建策略提供者
let policy = StaticPolicyProvider::new(CachePolicy::default()
    .with_namespace("user.profile")
    .with_ttl(Duration::from_secs(60))
    .with_tags(["user"]));

// 3. 安装缓存拦截器
rb.install(CacheIntercept::new(store.clone(), policy));
```

### 3.3 Axum 集成

```rust
use rbatis_plus::prelude::*;
use rbatis_plus_vernal::axum_integration::AppState;

let state = AppState::new(rb);
let app = Router::new()
    .route("/user/:id", get(get_user))
    .with_state(state);
```

---

## 4. 拦截器注册顺序

```text
1. SQL 重写拦截器（Pagination, Tenant, DynamicTableName）
2. 缓存拦截器（CacheIntercept）
3. 增强拦截器（EnhanceInnerInterceptor）
4. 观测拦截器（SqlObservation）
```

**关键规则：** 缓存拦截器必须放在重写拦截器之后、增强拦截器之前。

---

## 5. 条件构造器使用

### 5.1 QueryWrapper

```rust
let wrapper = QueryWrapper::new()
    .eq("name", "张三")
    .ge("age", 18)
    .and()
    .like("email", "@gmail.com")
    .order_by_asc("id")
    .last("LIMIT 10");

let sql = wrapper.build_select_sql("user");
// SELECT * FROM user WHERE name = '张三' AND age >= 18 AND email LIKE '%@gmail.com%' ORDER BY id ASC LIMIT 10
```

### 5.2 LambdaQueryWrapper

```rust
let wrapper = LambdaQueryWrapper::new()
    .eq(User::name, "张三")
    .ge(User::age, 18)
    .select(User::id, User::name);

let sql = wrapper.build_select_sql("user");
// SELECT id, name FROM user WHERE name = '张三' AND age >= 18
```

---

## 6. 分页使用

```rust
use rbatis_plus::page::{Page, PageRequest};

let page = PageRequest::new(1, 10);
let result: Page<User> = rb.select_page(&wrapper, "user", page).await?;
```

---

## 7. 事务使用

```rust
use rbatis_plus_vernal::transaction::TransactionalGuard;

let mut tx = rb.begin().await?;
// ... 操作
tx.commit().await?;

// 或使用 Guard 自动回滚
{
    let guard = TransactionalGuard::new(&rb).await?;
    // ... 操作
    // guard 在 Drop 时自动回滚
}
```

---

## 8. IService 使用

```rust
use rbatis_plus::service::{IService, ServiceImpl};

struct UserService {
    service: ServiceImpl<User>,
}

impl IService<User> for UserService {
    // ... 实现
}
```

---

## 9. 代码生成器使用

```rust
use rbatis_plus_generator::AutoGenerator;

let generator = AutoGenerator::new()
    .data_source(DataSourceConfig::new("mysql://..."))
    .package(PackageConfig::new("com.example"))
    .strategy(StrategyConfig::new()
        .include_tables(vec!["user", "order"]))
    .global(GlobalConfig::new()
        .output_dir("./generated"));

generator.generate().await?;
```

---

## 10. 兼容性矩阵

| rbatis-plus 版本 | rbatis 版本 | Rust 版本 |
|---|---|---|
| 0.1.x | master (fork) | >= 1.75 |
| 0.2.x | master (fork) | >= 1.75 |

---

## 11. 常见问题

### 11.1 缓存不生效

1. 检查拦截器注册顺序
2. 检查 feature flag `cache` 是否启用
3. 检查 `CachePolicy` 配置

### 11.2 事务内读取到旧数据

1. 检查 `TransactionCacheMode` 配置
2. 保守模式下事务内读取绕过缓存（预期行为）

### 11.3 拦截器不执行

1. 检查拦截器是否注册
2. 检查 `InterceptorIgnore` 注解
3. 检查拦截器顺序
