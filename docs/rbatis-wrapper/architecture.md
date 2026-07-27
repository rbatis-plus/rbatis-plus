# rbatis-wrapper 架构与代码导读

> 本文档基于本地仓库（`/Users/wandl/workspaces/workspace-github-easy-4-rust/rbatis-wrapper`）的**一手源码**梳理——这是 rbatis 生态中**极小型**的 wrapper crate，**整个项目只包含 1 个真实源文件**（`src/wrapper.rs`，310 行）。
>
> 参考资料：
> - GitHub：<https://github.com/362228416/rbatis-wrapper>
> - crates.io：<https://crates.io/crates/rbatis-wrapper>
> - 本地工作区：与 `rbatis` 同根（`workspace-github-easy-4-rust`）
> - 当前版本：`rbatis-wrapper 0.1.1`，依赖 `rbatis 4.6` + `serde 1`
> - License：MIT
>
> ---
>
> ## 在这里的原因
>
> 本文档路径：`rbatis-plus/docs/rbatis-wrapper/architecture.md`。
>
> 因为它是 **rbatis 生态中已有的链式查询构建器**——即使只是 1 个文件、9.5 KB 的体量，仍是一个**已经发布到 crates.io** 的工程，可以作为"在 rbatis 之上写 `BaseMapper<T>` 替代品"的极简参考实现：
>
> - 想在 Rust 里做"链式 `.eq(...).gt(...).like(...).order_by(...)`"？**本文档 §3 + §4** 直接给完整源码解读
> - 想把这个想法挂到 `rbatis-plus` 项目里（替换或补充 `crud!{}`）？**§7 移植路线**列了所有钩子
> - 想知道**9.5 KB 的单文件**如何实现 13+ 个链式 API + 分页 + Join + 自定义 SQL？请把本文档与 README 一起读

---

## 目录

1. 一句话定位与设计哲学
2. 仓库布局与规模（**1 文件 + 310 行**）
3. 数据结构：`QueryWrapper` + `Page<T>`
4. 5 个 WHERE 条件方法：eq/ne/gt/lt/like
5. 5 个 SQL 修饰方法：select/order_by/limit/offset
6. 3 个 JOIN：inner/left/right
7. 自定义 SQL + count 子查询：`custom_sql` / `build_count_sql`
8. 3 个执行入口：`query<T>` / `get_one<T>` / `page<T>` / `delete`
9. SQL 字符串拼接全流程（ASCII）
10. 与 MyBatis-Plus Wrapper / rbatis / rbatis-plus 对照速记表
11. 设计缺陷与已知坑（13 个）
12. 关键设计权衡（FAQ，10 条）
13. codegraph 速查命令
14. 推荐阅读顺序
15. 与 rbatis-plus 移植 checklist
16. 关联文档

---

## 1. 一句话定位与设计哲学

**rbatis-wrapper = "在 rbatis 4.6 之上做一个 MyBatis Plus 风格的链式查询构建器"。**

> "一个基于 rbatis 的现代化查询构建器，类似于 MyBatis Plus 的链式查询风格"——README

仓库作者：**CC（362228416@qq.com）**——非苞米豆官方，与 `rbatis-plus` 是平行项目。

### 1.1 与 rbatis 主仓的关系

- **`rbatis`** = 过程宏 + `Executor` trait + 拦截器链（**底座**）
- **`rbatis-wrapper`** = 在 `rbatis.query_decode / rb.exec` 之上的**链式 wrapper**，纯运行时（无过程宏）

二者是**正交关系**：
- 可以用 `wrapper.rs` 与 `crud!{}` 宏并存
- 也可以单独只用 wrapper（README 的"快速开始"示例就这样做）

### 1.2 一句话读完源码做的事

`QueryWrapper` 上调 `.eq(...) / .gt(...) / .like(...)` 等方法**只是把字符串塞进 `Vec<String>`**，`build_sql()` 时**用 `format!()` 拼出来**，然后调 `rb.query_decode(&sql, vec![])`。**没有 SQL 解析，没有安全转义，没有生成 AST，没有宏**——这是它能压到 9.5 KB 的根本原因，也是它**最大的安全责任**。

---

## 2. 仓库布局与规模

```
rbatis-wrapper/                            MIT, 单 crate
├── Cargo.toml                       ── rbatis 4.6 + serde 1
├── README.md                        ── 中文 API 文档（10+ 段代码示例）
├── LICENSE
└── src/
    ├── lib.rs                       ── 37 字节：pub mod wrapper; pub use wrapper::*;
    └── wrapper.rs                   ── 310 行
         ├── Page<T> 结构体 + impl
         └── QueryWrapper 结构体 + impl
              ├── 13 个链式 setter（self/mut self 混用）
              ├── build_sql()  ~ 65 行 字符串拼接
              ├── build_count_sql() ~ 30 行 count 子查询
              └── 4 个 async 执行方法
```

> **312 行 main 源**；git 提交历史 3 个 commit（`add delete` / `email` / `first commit`），活跃度极低。

---

## 3. 数据结构

### 3.1 `QueryWrapper`（59-68 行）

```rust
#[derive(Default, Debug, Clone)]
pub struct QueryWrapper {
    where_conditions: Vec<String>,        // sql 条件片段（已经手工拼接好）
    order_by: Vec<String>,               // "column ASC|DESC" 片段
    select_columns: Vec<String>,         // 指定列，空时为 *
    limit: Option<u64>,
    offset: Option<u64>,
    custom_sql: Option<String>,           // ★ 自定义 SQL 时，挤掉 select/where 的 prefix
    join_conditions: Vec<String>,        // "INNER JOIN ... ON ..." / "LEFT JOIN ..." / "RIGHT JOIN ..."
}
```

字段都是 `private`，**所有 setter 通过 `self/mut self` 链式修改再返回**——这是为 `QueryWrapper::new().eq(...).gt(...)` 风格而设计的"建造者"模式。

### 3.2 `Page<T>`（7-30 行）

```rust
#[derive(Debug, Serialize)]
pub struct Page<T> {
    pub records: Vec<T>,         // 数据列表
    pub total: u64,             // 总记录数
    pub page_no: u64,           // 当前页码
    pub page_size: u64,         // 每页大小
    pub pages: u64,             // 总页数（推导：ceil(total/page_size)）
    pub has_next: bool,         // page_no < pages
}

impl<T> Page<T> {
    pub fn new(records: Vec<T>, total: u64, page_no: u64, page_size: u64) -> Self { ... }
}
```

`pages` 用 `(total + page_size - 1) / page_size` 直接公式推——**核心整数算法**。

### 3.3 一个**潜在的设计不一致**：return `Self` vs `&mut Self`

源码中 setter 混用了 4 种签名（按出现顺序）：

| 方法 | 签名 | 类型 |
|---|---|---|
| `new` / `eq` / `ne` / `gt` / `lt` / `like` / `select` / `order_by` / `custom_sql` / `inner_join` / `left_join` / `right_join` | `mut self -> Self` | **消费-返回** |
| `limit` / `offset` | `&mut self -> &mut Self` | **可变引用** |

**两种风格混用意味着**：要继续链式调用，必须在 `limit/offset` 之后断链：

```rust
QueryWrapper::new()
    .limit(20)               // mut self -> Self（**断链**）
    .offset(0)
    .eq("status", 1);        // ← 这一行就不能再用链式了，因为 limit 返回 &mut Self
```

更糟糕的是 `page<T>` 内部：

```rust
let mut wrapper = self.clone();
wrapper.limit(page_size);   // 必须先 clone + 不能接方法
wrapper.offset(offset);     // 不能接方法
let records = wrapper.query(rb, table_name).await?;  // 只能用 query
```

**这是这个 crate 最大的"使用不便点"**——详见 §11 缺陷。

---

## 4. 5 个 WHERE 条件方法

```rust
pub fn eq<T: ToString>(mut self, column: &str, value: T) -> Self {
    self.where_conditions.push(format!("{} = '{}'", column, value.to_string()));
    self
}
pub fn ne<T: ToString>(mut self, column: &str, value: T) -> Self {
    self.where_conditions.push(format!("{} != '{}'", column, value.to_string()));
    self
}
pub fn gt<T: ToString>(mut self, column: &str, value: T) -> Self {
    self.where_conditions.push(format!("{} > '{}'", column, value.to_string()));
    self
}
pub fn lt<T: ToString>(mut self, column: &str, value: T) -> Self {
    self.where_conditions.push(format!("{} < '{}'", column, value.to_string()));
    self
}
pub fn like(mut self, column: &str, value: &str) -> Self {
    self.where_conditions.push(format!("{} LIKE '%{}%'", column, value));
    self
}
```

> ⚠️ **致命问题**：所有值都用 `format!("'{}'", value)` 包成字符串再插入。**没有 SQL injection 防护**。
>
> ```rust
> .eq("name", "'; DROP TABLE users; --")
> // 最终 SQL: WHERE name = '''; DROP TABLE users; --'  ←  原样插入，没人转义
> ```
>
> 还有：值用 `single quote` 包，**含单引号的字符串**（`O'Brien`）会被截断；**整数/NULL/Option<T>** 没区分（直接 to_string）。

详见 §11。

---

## 5. 5 个 SQL 修饰方法

```rust
pub fn select(mut self, columns: Vec<&str>) -> Self {
    self.select_columns = columns.into_iter().map(String::from).collect();
    self
}

pub fn order_by(mut self, column: &str, asc: bool) -> Self {
    let order = if asc { "ASC" } else { "DESC" };
    self.order_by.push(format!("{} {}", column, order));
    self
}

pub fn limit(&mut self, limit: u64) -> &mut Self {
    self.limit = Some(limit);
    self
}

pub fn offset(&mut self, offset: u64) -> &mut Self {
    self.offset = Some(offset);
    self
}
```

注意 `limit/offset` 用 `&mut self` —— README 用例：

```rust
let mut wrapper = QueryWrapper::new();
wrapper
    .limit(20)
    .offset(0);
let users = wrapper
    .eq("department", "技术部")     // 这里断链了（先 mut，再 eq）
    ...
```

——除非你接受"分页相关的 limit/offset 与查询构建的 set 链路不能合并为一条链"。**这种混合 API 是有意为之还是 bug？** → 见 §11 FAQ Q5。

---

## 6. 3 个 JOIN 方法

```rust
pub fn inner_join(mut self, table: &str, on_condition: &str) -> Self {
    self.join_conditions.push(format!("INNER JOIN {} ON {}", table, on_condition));
    self
}
pub fn left_join(mut self, table: &str, on_condition: &str) -> Self {
    self.join_conditions.push(format!("LEFT JOIN {} ON {}", table, on_condition));
    self
}
pub fn right_join(mut self, table: &str, on_condition: &str) -> Self {
    self.join_conditions.push(format!("RIGHT JOIN {} ON {}", table, on_condition));
    self
}
```

`on_condition` 直接拼字符串。**完全不做列名前缀校验、表名白名单**——用户责任。

---

## 7. custom_sql + build_count_sql

### 7.1 `custom_sql("SELECT * FROM users")` 路径

```rust
pub fn custom_sql(mut self, sql: &str) -> Self {
    self.custom_sql = Some(sql.to_string());
    self
}
```

`build_sql()` 检测到 `custom_sql` 时不走默认 SELECT：

```rust
pub fn build_sql(&self, table_name: &str) -> String {
    if let Some(custom_sql) = &self.custom_sql {
        let mut sql = custom_sql.clone();
        if !self.where_conditions.is_empty() {
            if !sql.to_uppercase().contains("WHERE") {
                sql.push_str(" WHERE ");
            } else {
                sql.push_str(" AND ");
            }
            sql.push_str(&self.where_conditions.join(" AND "));
        }
        if !self.order_by.is_empty() { sql.push_str(" ORDER BY "); ... }
        if let Some(limit) = self.limit { sql.push_str(&format!(" LIMIT {}", limit)); }
        if let Some(offset) = self.offset { sql.push_str(&format!(" OFFSET {}", offset)); }
        return sql;
    }
    // ... 默认 SELECT 路径
}
```

> **实现细节**：检测 "WHERE" 的关键字用 `sql.to_uppercase()` 然后 `contains("WHERE")`——无法区分 `WHERE` 是不是字符串里的字面量。SQL 字面量写法 `WHERE 'no_where_substring'` 会误判。**实际场景通常不冲突**。

### 7.2 `build_count_sql` 为分页 count 服务

`page<T>` 内会先发一次 `count_sql`：

```rust
fn build_count_sql(&self, table_name: &str) -> String {
    if let Some(custom_sql) = &self.custom_sql {
        let mut inner_sql = custom_sql.clone();
        if !self.where_conditions.is_empty() {
            // 同 build_sql，附加 WHERE
        }
        format!("SELECT COUNT(*) FROM ({}) as t", inner_sql)   // ★ 包成子查询
    } else {
        let mut sql = format!("SELECT COUNT(*) FROM {}", table_name);
        // ... JOIN / WHERE 同样 append
        sql
    }
}
```

`format!("SELECT COUNT(*) FROM ({}) as t", inner_sql)` —— 把整个 custom sql 包成子查询。

**问题**：`custom_sql` 是 `SELECT * FROM users WHERE age > 18` 时，再装一层 `( ... )` 等于 `SELECT COUNT(*) FROM (SELECT ... FROM users WHERE ...) as t` —— 没问题。但如果 custom_sql 已经是 `SELECT id, name FROM users GROUP BY name` —— count 会变成 "不同 name 个数"，**不是"记录行数"**。

---

## 8. 3 个执行入口 + delete

### 8.1 `query<T>(rb, table_name)`

```rust
pub async fn query<T>(&self, rb: &RBatis, table_name: &str) -> Result<Vec<T>, Error>
where
    T: Serialize + for<'de> serde::Deserialize<'de>,
{
    let sql = self.build_sql(table_name);
    rb.query_decode(&sql, vec![]).await
}
```

### 8.2 `get_one<T>(rb, table_name)`

```rust
pub async fn get_one<T>(&self, rb: &RBatis, table_name: &str) -> Result<Option<T>, Error>
where
    T: Serialize + for<'de> serde::Deserialize<'de>,
{
    let sql = self.build_sql(table_name);
    rb.query_decode::<Option<T>>(&sql, vec![]).await
}
```

> 直接让 rbatis 解 `Option<T>`——空则返回 `Ok(None)`，非空则返回 `Ok(Some(t))`。

### 8.3 `page<T>(rb, table_name, page_no, page_size)`

```rust
pub async fn page<T>(&self, rb: &RBatis, table_name: &str, page_no: u64, page_size: u64) -> Result<Page<T>, Error>
where T: Serialize + for<'de> serde::Deserialize<'de> {
    let count_sql = self.build_count_sql(table_name);
    let total: u64 = rb.query_decode(&count_sql, vec![]).await?;
    if total > 0 {
        let offset = (page_no - 1) * page_size;
        let mut wrapper = self.clone();
        wrapper.limit(page_size);
        wrapper.offset(offset);
        let records: Vec<T> = wrapper.query(rb, table_name).await?;
        Ok(Page::new(records, total, page_no, page_size))
    } else {
        Ok(Page::new(vec![], 0, page_no, page_size))
    }
}
```

**做两件事**：
1. 一次 `count_sql` → 拿 `total`
2. `total > 0` 才查 records（避免空集分页浪费一次 DB）
3. `self.clone()` + `wrapper.limit/offset`（因为 `limit/offset` 用 `&mut self` 而 `self` 已经是 `&self`）

### 8.4 `delete(rb, table_name)`

```rust
pub async fn delete(self, rb: &RBatis, table_name: &str) -> Result<u64, Error> {
    let delete_sql = format!("delete from {}", table_name);
    let sql = self.custom_sql(&delete_sql).build_sql(table_name);   // ★ 注意
    Ok(rb.exec(&sql, vec![]).await?.rows_affected)
}
```

**注意一个隐含的 hack**：

1. `delete` 体内调用 `self.custom_sql(&delete_sql)`——`self` 是 `mut self`，`custom_sql` 也是 `mut self -> Self`
2. 但没有 `mut self` 在签名上（签名是 `self`）—— **Rust 自动 mutability** 因为 `self` 形参
3. 最后调 `self.custom_sql(...)` 后 `self` 已被消费，无法再用 `.eq(...)` 等链式

——也就是说，**`delete` 后无法加任何 WHERE / ORDER**！源码就这么限定了。

**改进路径**：让 `delete` 也有 chain API，**直接调内 `build_sql` 套路**：

```rust
pub async fn delete_with_where(self, rb: &RBatis, table_name: &str) -> Result<u64, Error> {
    let mut sql = format!("DELETE FROM {}", table_name);
    if !self.where_conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&self.where_conditions.join(" AND "));
    }
    Ok(rb.exec(&sql, vec![]).await?.rows_affected)
}
```

——这是 §12 Q3 要讨论的设计权衡。

---

## 9. SQL 字符串拼接全流程

```
                       user code
                          │
                          ▼
                  QueryWrapper::new()
                          │
                          ▼
        ┌──────────── .eq( ... )  ←──────────┐
        │  where_conditions.push("col='v'") │  ←── 不带 escaping
        ├──────────── .gt( ... )  ←───────────┤
        │  where_conditions.push("col > 'v'")│
        ├──────────── .like( ... )  ←─────────┤
        │  where_conditions.push("c LIKE %v%")│
        ├──────────── .select(vec!["id","name"]) ─ select_columns = ["id","name"]
        ├──────────── .order_by("created_at", false) ─ order_by = ["created_at DESC"]
        ├──────────── .inner_join("profiles p", "u.id = p.user_id") ─ join_conditions = ["INNER JOIN ..."]
        ├──────────── .custom_sql("SELECT id FROM users") ─ custom_sql = Some("SELECT id FROM users")
        ├──────────── .limit(20)                ← 注意 mut self vs self
        └──────────── .offset(40)                
                          │
                          ▼
                  .query::<User>(&rb, "users u")
                          │
                          ▼
                build_sql("users u")            
                          │
                          ▼  (custom_sql == None)
SELECT id, name FROM users u INNER JOIN profiles p ON ... WHERE col='v' AND col > 'v' AND c LIKE %v% ORDER BY created_at DESC LIMIT 20 OFFSET 40
                          │
                          ▼
              rb.query_decode::<Vec<User>>(sql, vec![])
                          │
                          ▼
                       Result<Vec<User>, Error>
```

如果是 `page`：

```
.page::<User>(&rb, "users", 1, 10)
        │
        ├──▶ build_count_sql → "SELECT COUNT(*) FROM users WHERE ..." → rb.query_decode::<u64>(...) → total
        ├──▶ self.clone() + wrapper.limit(10).offset(0) + .query(rb, table_name) → records
        └──▶ Page::new(records, total, 1, 10)
```

---

## 10. 与 MyBatis-Plus / rbatis / rbatis-plus 对照速记表

| 维度 | **rbatis-wrapper** | MyBatis-Plus `Wrapper<T>` | rbatis 主仓 `crud!{}` | rbatis-plus 缺 |
|---|---|---|---|---|
| 大小 | **9.5 KB / 1 文件** | 数千行 Java | 过程宏 + 编译期 | 待做 |
| 链式 API | ✓ | ✓ | `value!{...}` map | 期望能链式 |
| 字段名类型安全 | **✗（字符串）** | ✓（`SFunction<T,?>` 反射） | 部分（`#[snake_name]`） | — |
| SQL injection | **✗（无转义）** | 反射 + 模板安全 | 编译期绑定 | — |
| 条件方法数 | 5（eq/ne/gt/lt/like） | 30+（between / in / notIn / likeLeft / likeRight / ...） | 12（`crud_traits.rs`） | — |
| JOIN | 3（inner/left/right） | 通过 lambda + 字符串 | 编译期宏可支持 | — |
| 分页 | ✓（2 次查询 count + data） | ✓（inner interceptor 自动 count） | `PagePlugin` | 待做 |
| 自定义 SQL | ✓（`custom_sql`） | `Wrapper.apply(...)` | `py_sql` / `html_sql` | — |
| 主键支持 | ✗ | ✓（`TableInfo.havePK()`） | 自动 | — |
| 物理删除 / 逻辑删除 | ✗ | ✓（`@TableLogic`） | 部分 | — |
| 事务监听 | ✗ | ✓（`@InterceptorIgnore` 等） | `TransactionListener` | — |
| 缓存集成 | ✗ | 可用 MyBatis L2 | ✓（已合入 `df87ac41`） | ✓（主仓） |
| License | MIT | Apache-2.0 | Apache-2.0 | — |

——`rbatis-wrapper` 的价值在于**极简**和**直白**，而不是**生产可用**。

---

## 11. 设计缺陷与已知坑（13 个，按严重程度）

### 严重（务必修）— 4 个

1. **SQL injection 无防护**：`format!("'{}'", value)` 直接拼字符串，输入 `'; DROP TABLE users; --` 是裸 SQL 注入
2. **字符串引号截断**：值里含单引号（`O'Brien`）会被截断到 `'O'` 之后的部分
3. **`Option` / `null` / 数字未区分**：所有类型都走 `value.to_string()` —— `None` 会出 `Option<None>`、`Vec<u8>` 会出字节字符串
4. **字段名无校验**：列名由用户字符串提供 —— 拼成无效 SQL 时 DB 方报错信息不一定好看

### 中等（影响使用）— 5 个

5. **`limit/offset` 用 `&mut self`**：破坏链式风格；与 `eq/gt/select` 风格不一致
6. **`delete` 无 WHERE 链**：源码写死 `delete from <table>`，无法追条件
7. **`page<T>` 内部会重新算 SQL**：当用户 `custom_sql("SELECT a FROM ...")` + JOIN 时，count 与 data 走两条 SQL，需要测试一致性
8. **`Page` 内存全量计算**：`has_next = page_no < pages` 没考虑 `page_no == 0` 入参，0 永远小于 pages → `has_next = true`
9. **`has_next` 没考虑 `page_no == pages` + 数据被删越界的情况**

### 轻微（边缘 bug）— 4 个

10. **`custom_sql` 大写判断 `to_uppercase().contains("WHERE")`**：误判可能（如 SQL 字面量"WHERE")
11. **`Page::new` 没声明 `serde::Deserialize`** —— `Page<T>` 只 derive 了 `Serialize`，无法 round-trip；分页响应若想反序列化，必须自己写
12. **rust-toolchain**：仓库 `Cargo.toml` 仅 `edition = "2024"`，无 `rust-version` —— 不强制最低 Rust 版本（实际需要 1.80+）
13. **依赖仅 1 个版本的 rbatis（`4.6`）**：未与上游 `rbatis 4.9.x`（已合入 Caffeine 化 `df87ac41`）对齐

### 性能（trait-allocation）— 1 个

> `[T: Serialize + Deserialize<'de>]` —— 编译期单态化，**无运行时 trait object**，这是 rbatis 的优点。整个 wrapper.rs 也没有运行时开销。

---

## 12. 关键设计权衡（FAQ）

### Q1：为什么只有 1 个文件、9.5 KB？是不是写得太简陋？

A: 因为它**只做"字符串拼接 + 透传 SQL"**。rbatis 主仓提供了：
- `rb.query_decode(sql, args)` —— 直接传 SQL 字符串
- `serde` 解码 `Option<T> / Vec<T> / T` 全自动

**所以 `rbatis-wrapper` 是"壳 API"**。它没造 AST、没造 SQL 解析、没造注入检测——那是 sqlparser/sqllogictest 库的工作。

### Q2：链式 `Self` vs `&mut Self` 混用是不是 bug？

源码直接读：

- `mut self -> Self`：新增字段 → 用 `mut self`
- `&mut self -> &mut Self`：累加字段 → 用 `&mut self`

**这是 Rust 中"建造者模式"经常踩的坑**。Rust 1.74+ 起 `Receiver` 模式可以让所有 setter 自动用 `mut self`，但本 crate 没有引入新语法。

**风格一致性建议**：要么全部改回 `mut self -> Self`，要么全部改为 `&mut self -> &mut Self`。混合是最差的中间方案。

### Q3：`delete` 没法链式增 WHERE 应该怎么修？

源码第 242-247 行的 `delete` 是一行误导代码：

```rust
let delete_sql = format!("delete from {}", table_name);
let sql = self.custom_sql(&delete_sql).build_sql(table_name);
```

如果有人想 `wrapper.eq("status", 0).delete(...)`，**第一个 .eq 把 self 消费掉，delete 就拿不到这些 condition**——SQL 只能是 `delete from <table>`。

修复方法（**不在源码中，但在 §15 checklist 里会进 rbatis-plus**）：

```rust
pub async fn delete_chain(self, rb: &RBatis, table_name: &str) -> Result<u64, Error> {
    let where_sql = if self.where_conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", self.where_conditions.join(" AND "))
    };
    let sql = format!("DELETE FROM {}{}", table_name, where_sql);
    Ok(rb.exec(&sql, vec![]).await?.rows_affected)
}
```

### Q4：怎么做参数化绑定？

最简单的方式：**别用这个 crate 拼字符串**。直接调：

```rust
rb.query_decode("SELECT * FROM users WHERE id = ?", vec![rbs::Value::I64(1)]).await?
```

——rbatis 用 `?` 占位 + `Vec<rbs::Value>` 传参，自动用 JDBC PreparedStatement 参数化。**这是官方推荐模式**。

`rbatis-wrapper` 显然不打算这么改。

### Q5：链式风格如何对齐 (推荐改法)

如果保留 `&mut self` API：

```rust
QueryWrapper::new()
    .eq("status", 1)
    .mut_self()       // 一个 .mut_self() 把 &mut 语义统一
    .limit(20)
    .offset(0)
```

但实际**标准做法**是：所有 setter 都用 `mut self -> Self`。把 `limit/offset` 也改成 `mut self`。

### Q6：为什么 `Page` 不 derive `serde::Deserialize`？

依赖 `serde::Serialize` 即可让响应序列化。但**反序列化要求 `T: Deserialize`**，`Page<T>::deserialize` 不能直接 derive。**原因**：Page 的 `has_next` 是 derived 字段，不能被外部覆盖。手动写 impl 也可以（不严格）。

### Q7：`Option<T>` 透传为什么 OK？

```rust
rb.query_decode::<Option<T>>(&sql, vec![]).await
```

rbatis 的 `decode.rs` 知道怎么处理空集合 vs 单元素。详见 `rbatis/docs/rbatis-architecture.md` §6。

### Q8：`page<T>` 在 `total = 0` 时为什么返回 `Page::new(vec![], 0, ...)` 而不是 `Page::new(vec![], ...)`？

`total = 0` 时 `records` 就是 `vec![]`，`page_no < pages` 是 `page_no < 0` = `false`——`has_next = false`。这样**实现正确**。

**有 bug 的地方**：如果 `page_no = 0`、`page_size = 10`、`total = 100`，会 `has_next = (0 < 10) = true`——**但 page_no 从 1 开始**才合理。本 crate 默认 `page_no` 从 1 开始（`(page_no - 1) * page_size` 当 offset），但没校验入参。

### Q9：为什么用 `Vec<String>` 而不是 `Vec<Condition>`？

A: 因为不想造 Condition enum + impl trait。**这是极简的代价**。

如果换更强类型：

```rust
pub enum Condition {
    Eq(String, Value),
    Gt(String, Value),
    ...
}
pub where_conditions: Vec<Condition>;
```

→ 解析与转义都能集中到 `to_sql()` 一个方法。但**仍然是字符串拼接**——除非借 sqlparser 这类库做完整解析。

### Q10：与 rbatis-plus 的差异

`rbatis-plus`（在 `rbatis` 旁的另一个 crate，**未来**要做的事）需要：
- 字段名类型安全（refactor 到 `Column<T>` enum + serde 自动映射）
- SQL 注入安全（要么绑定参数、要么 sqlparser AST 输出）
- 全分页支持（先 count 再 SELECT，与本实现一致）
- 主键 / 逻辑删除 / 乐观锁（参考 MyBatis-Plus 的 `TableInfo` 缓存）
- 监听器链：MybatisPlusInterceptor 已经实现 Rust 版本叫 `Intercept` trait

——本 crate 可以作为"最简实现"，不作为"目标实现"。

---

## 13. codegraph 速查命令

```bash
export PATH="/Users/wandl/.nvm/versions/node/v24.18.0/bin:$PATH"
# 项目极小，不建议初始化 codegraph；直接读源码即可
```

如果要看 workspace-github-easy-4-rust 全局：

```bash
cd /Users/wandl/workspaces/workspace-github-easy-4-rust
codegraph status
codegraph query "QueryWrapper\|pub fn eq\|pub fn like\|pub fn page"
```

---

## 14. 推荐阅读顺序

1. **`Cargo.toml`** —— 看依赖只有 `rbatis 4.6 + serde`
2. **`src/lib.rs`** —— 37 字节
3. **`src/wrapper.rs::Page<T>`**（30 行）—— 分页数据结构
4. **`src/wrapper.rs::QueryWrapper::new()` 到 `eq/ne/gt/lt/like`**（前 105 行）—— 5 种条件 setter
5. **`select/order_by/limit/offset/custom_sql`**（107-134 行）—— SQL 修饰
6. **`inner_join/left_join/right_join`**（137-152 行）—— JOIN
7. **`build_sql`**（155-221 行）—— ★ 主分发：所有策略收敛到一段 `format!`
8. **`query/get_one`**（223-239 行）—— Rust async 入口
9. **`page<T>`**（250-274 行）—— ★ ★ 分页 2-query 流程
10. **`delete`**（242-247 行）—— 注意这里 `custom_sql` 替换了之前的 custom_sql
11. **`build_count_sql`**（277-309 行）—— count 子查询

---

## 15. 与 `rbatis-plus` 移植 checklist

如果想在 `workspace-github-easy-4-rust/rbatis-plus/docs/../rbatis-plus/` 目录下做一个"rbatis 风格 MyBatis-Plus"，**`rbatis-wrapper` 这个 9.5 KB 单文件是起点**，但必须**几乎完全重写**：

| 维度 | 当前 rbatis-wrapper | rbatis-plus 应有 |
|---|---|---|
| 字段名 | 字符串 | `Column<T>` enum 派生 + 反射 |
| 参数 | `'val'` 包字符串 | `Vec<rbs::Value>` 绑定参数 |
| LIKE | `LIKE '%v%'` 字符串 | 字符串拼接（无害）或用 jsqlparser 替换 |
| 条件方法 | 5 个 | 30+（gt/lt/ge/le/between/notBetween/in/notIn/likeLeft/...） |
| count + page | 2 query | 2 query + 缓存 `total` count |
| JOIN | 3 种 + on_condition 字符串 | 与 SQL 一样严谨验证 |
| 主键 / 逻辑删除 | ✗ | 反射元数据缓存（仿 MyBatis-Plus `TableInfo`） |
| 自定义 SQL | `custom_sql` 字符串 | `py_sql!{}` 宏（rbatis 已有） |
| 类型安全 | ✗ | `T::Column: Column` 约束 |
| 事务 | ✗ | 复用 `TransactionListener` |
| 缓存 | ✗ | 复用主仓 `Intercept` + `MemoryCacheStore` |

——**这个对照 = rbatis-plus 的雏形**。

---

## 16. 关联文档（位于 `rbatis-plus/docs/`）

- `../mybatis-plus-architecture.md`（**872 行**）—— MyBatis-Plus 主框架的 Java 版参考
- `../mybatis-plus-enhance/architecture.md`（**999 行**）—— MyBatis-Plus 之上的第三方企业级增强
- `../mybatis-3/architecture.md`（如存在）—— MyBatis 3 主流程
- `../../rbatis/docs/rbatis-architecture.md`（**838 行**）—— rbatis（Rust 主仓）自家文档
- `../../rbatis-cache/docs/rbatis-cache-architecture.md`（**740 行**）—— rbatis-cache SPI

> **建议阅读顺序**（与 `rbatis-plus` 设计议题相关）：
>
> 1. `rbatis-wrapper/architecture.md`（**本文，起点**）—— Rust 链式 wrapper 的极简可行性 + 安全欠账
> 2. `mybatis-plus-architecture.md` —— MyBatis-Plus Wrapper 的 30 个条件方法 + Lambda 反射 + 元数据缓存
> 3. `rbatis-architecture.md` —— rbatis 已有 `crud!{}` 宏、Executor trait、`Intercept` 拦截链
> 4. 汇总：在 `rbatis-plus/docs/INDEX.md` 或 `DECISIONS.md` 中给出"是不是把 rbatis-wrapper 当起点 + 重大改造"决策记录

---

## 附：`rbatis-wrapper` 完整源码逐行注释版（精简）

篇幅允许的话，给整个 310 行加注释会更有用——但本文档已经解释了核心 80% 设计点。如果你要更深入，**直接读源文件 +本文档对照**比读更详细的"逐行注释"更好（避免文档与代码 drift）。

> 直接读源码 + 跟着本文档的章节顺序对照，是最快学会"在 rbatis 上加一个 wrapper"的方法。
