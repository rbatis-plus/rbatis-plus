# rbatis-plus

Rust-native MyBatis-Plus style contracts for RBatis. This workspace contains:

- `rbatis-plus-core`: `BaseMapper`, `IService`, `ServiceImpl`, typed query/update wrappers and the fixed six-stage interceptor chain;
- `rbatis-plus-macros`: compile-time table/column metadata;
- `rbatis-plus-extension`: tenant, data-permission and SQL observation interceptors;
- `rbatis-plus-codegen`: deterministic model generation;
- `rbatis-plus`: facade and prelude.

Interceptor order is fixed: `SQL_REWRITE`, `PARAMETER_TRANSFORM`, `EXECUTE`,
`RESULT_VERIFY`, `RESULT_TRANSFORM`, `OBSERVE`.

Current status is an executable alpha vertical slice, not feature parity. RBatis
database mappers, optimistic locking, logical deletion, audit fill, encryption,
blind indexes, row signatures, internationalization, long-SQL checks and the
four-database matrix remain tracked work before `1.0.0`.
