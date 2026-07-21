# rbatis-plus

Rust-native MyBatis-Plus style contracts for RBatis. This workspace contains:

- `rbatis-plus-core`: `BaseMapper`, `IService`, `ServiceImpl`, typed query/update wrappers and the fixed six-stage interceptor chain;
- `rbatis-plus-macros`: compile-time table/column metadata;
- `rbatis-plus-extension`: native RBatis mapper plus tenant, data-permission and SQL observation interceptors;
- `rbatis-plus-codegen`: deterministic model generation;
- `rbatis-plus`: facade and prelude.

Interceptor order is fixed: `SQL_REWRITE`, `PARAMETER_TRANSFORM`, `EXECUTE`,
`RESULT_VERIFY`, `RESULT_TRANSFORM`, `OBSERVE`.

Current status is an executable alpha vertical slice, not feature parity. The
native `RbatisMapper` executes CRUD, atomic batch insert, pagination, typed
query/update wrappers, optimistic locking and logical deletion. Audit fill,
internationalization, long-SQL checks and the PostgreSQL/MySQL/SQLite/SQL Server
matrix remain tracked work before `1.0.0`.

The security vertical slice provides versioned AES-256-GCM field envelopes,
context-bound blind indexes, HMAC row signatures, key rotation,
`REJECT_PARTIAL`/`DEFERRED_RESIGN`, and fixed `RESULT_VERIFY` before
`RESULT_TRANSFORM` ordering. The provider API remains open for an SM4/SM3
compatibility implementation. `RbatisMapper::with_interceptors` applies rewrite
and parameter stages before execution, then verification, transformation and
observation stages to raw query rows before model deserialization.
