use futures::future::BoxFuture;
use rbatis::RBatis;
use rbatis::executor::Executor;
use rbatis_plus_core::{
    BaseMapper, InterceptorChain, Operator, Page, PageRequest, PlusError, PlusResult, QueryWrapper,
    SortDirection, SqlInvocation, TableMetadata, UpdateWrapper,
};
use rbs::Value;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;
use std::fmt::Write as _;
use std::marker::PhantomData;
use std::sync::Arc;

/// A native `RBatis` implementation of the MyBatis-Plus-style `BaseMapper`.
#[derive(Clone)]
pub struct RbatisMapper<T, Id> {
    rbatis: RBatis,
    interceptors: Option<Arc<InterceptorChain>>,
    marker: PhantomData<fn() -> (T, Id)>,
}

impl<T, Id> RbatisMapper<T, Id>
where
    T: TableMetadata + Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    Id: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub fn new(rbatis: RBatis) -> PlusResult<Self> {
        validate_metadata::<T>()?;
        Ok(Self {
            rbatis,
            interceptors: None,
            marker: PhantomData,
        })
    }

    #[must_use]
    pub fn with_interceptors(mut self, interceptors: Arc<InterceptorChain>) -> Self {
        self.interceptors = Some(interceptors);
        self
    }

    pub fn rbatis(&self) -> &RBatis {
        &self.rbatis
    }

    async fn insert_on(&self, executor: &dyn Executor, entity: &T) -> PlusResult<()> {
        let value = model_value(entity)?;
        let columns = T::COLUMNS.join(", ");
        let placeholders = vec!["?"; T::COLUMNS.len()].join(", ");
        let args = T::COLUMNS
            .iter()
            .map(|column| field(&value, column))
            .collect::<PlusResult<Vec<_>>>()?;
        let sql = format!(
            "INSERT INTO {} ({columns}) VALUES ({placeholders})",
            T::TABLE_NAME
        );
        self.execute_on(executor, "BaseMapper.insert", sql, args)
            .await?;
        Ok(())
    }

    async fn select_one_by_value(&self, id: Value) -> PlusResult<Option<T>> {
        let mut sql = format!(
            "SELECT {} FROM {} WHERE {} = ?",
            T::COLUMNS.join(", "),
            T::TABLE_NAME,
            T::ID_COLUMN
        );
        append_logic_filter::<T>(&mut sql);
        let rows: Vec<T> = self
            .query_decode_on(&self.rbatis, "BaseMapper.selectById", sql, vec![id])
            .await?;
        Ok(rows.into_iter().next())
    }

    async fn prepare(
        &self,
        statement_id: &str,
        sql: String,
        args: Vec<Value>,
    ) -> PlusResult<SqlInvocation> {
        let parameters = args
            .into_iter()
            .map(|value| serde_json::to_value(value).map_err(mapper_error))
            .collect::<PlusResult<Vec<_>>>()?;
        let mut invocation = SqlInvocation::new(statement_id, sql, parameters);
        if let Some(interceptors) = &self.interceptors {
            interceptors.apply_before_execute(&mut invocation).await?;
        }
        Ok(invocation)
    }

    async fn finish(&self, invocation: &mut SqlInvocation) -> PlusResult<()> {
        if let Some(interceptors) = &self.interceptors {
            interceptors.apply_after_execute(invocation).await?;
        }
        Ok(())
    }

    async fn execute_on(
        &self,
        executor: &dyn Executor,
        statement_id: &str,
        sql: String,
        args: Vec<Value>,
    ) -> PlusResult<u64> {
        let mut invocation = self.prepare(statement_id, sql, args).await?;
        let args = invocation_args(&invocation)?;
        let result = executor
            .exec(&invocation.sql, args)
            .await
            .map_err(mapper_error)?;
        self.finish(&mut invocation).await?;
        Ok(result.rows_affected)
    }

    async fn query_decode_on<R: DeserializeOwned>(
        &self,
        executor: &dyn Executor,
        statement_id: &str,
        sql: String,
        args: Vec<Value>,
    ) -> PlusResult<R> {
        let mut invocation = self.prepare(statement_id, sql, args).await?;
        let args = invocation_args(&invocation)?;
        let result = executor
            .query(&invocation.sql, args)
            .await
            .map_err(mapper_error)?;
        invocation.result = Some(serde_json::to_value(result).map_err(mapper_error)?);
        self.finish(&mut invocation).await?;
        serde_json::from_value(
            invocation
                .result
                .ok_or_else(|| PlusError::Mapper("query result was removed".to_owned()))?,
        )
        .map_err(mapper_error)
    }
}

impl<T, Id> BaseMapper<T, Id> for RbatisMapper<T, Id>
where
    T: TableMetadata + Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    Id: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    fn insert(&self, entity: T) -> BoxFuture<'_, PlusResult<T>> {
        Box::pin(async move {
            self.insert_on(&self.rbatis, &entity).await?;
            Ok(entity)
        })
    }

    fn select_by_id(&self, id: Id) -> BoxFuture<'_, PlusResult<Option<T>>> {
        Box::pin(async move {
            self.select_one_by_value(rbs::to_value(id).map_err(mapper_error)?)
                .await
        })
    }

    fn select_list(&self, query: QueryWrapper<T>) -> BoxFuture<'_, PlusResult<Vec<T>>> {
        Box::pin(async move {
            let (where_clause, mut args) = compile_where::<T>(&query)?;
            let mut sql = format!(
                "SELECT {} FROM {}{where_clause}",
                T::COLUMNS.join(", "),
                T::TABLE_NAME
            );
            append_order::<T>(&mut sql, &query)?;
            if let Some(limit) = query.row_limit() {
                sql.push_str(" LIMIT ?");
                args.push(Value::U64(limit));
            }
            self.query_decode_on(&self.rbatis, "BaseMapper.selectList", sql, args)
                .await
        })
    }

    fn select_page(
        &self,
        page: PageRequest,
        query: QueryWrapper<T>,
    ) -> BoxFuture<'_, PlusResult<Page<T>>> {
        Box::pin(async move {
            let (where_clause, args) = compile_where::<T>(&query)?;
            let count_sql = format!("SELECT COUNT(*) FROM {}{where_clause}", T::TABLE_NAME);
            let mut count_invocation = self
                .prepare("BaseMapper.selectPage.count", count_sql, args.clone())
                .await?;
            let count_args = invocation_args(&count_invocation)?;
            let total: i64 = self
                .rbatis
                .exec_decode(&count_invocation.sql, count_args)
                .await
                .map_err(mapper_error)?;
            self.finish(&mut count_invocation).await?;
            let total = u64::try_from(total).map_err(|_| {
                PlusError::Mapper(format!("database returned a negative row count: {total}"))
            })?;
            let mut sql = format!(
                "SELECT {} FROM {}{where_clause}",
                T::COLUMNS.join(", "),
                T::TABLE_NAME
            );
            append_order::<T>(&mut sql, &query)?;
            sql.push_str(" LIMIT ? OFFSET ?");
            let mut page_args = args;
            page_args.push(Value::U64(page.size));
            page_args.push(Value::U64((page.current - 1) * page.size));
            let records = self
                .query_decode_on(
                    &self.rbatis,
                    "BaseMapper.selectPage.records",
                    sql,
                    page_args,
                )
                .await?;
            Ok(Page {
                records,
                total,
                current: page.current,
                size: page.size,
            })
        })
    }

    fn update_by_id(&self, entity: T) -> BoxFuture<'_, PlusResult<T>> {
        Box::pin(async move {
            let value = model_value(&entity)?;
            let id_value = field(&value, T::ID_COLUMN)?;
            let id: Id = rbs::from_value(id_value.clone()).map_err(mapper_error)?;
            let assignments = T::COLUMNS
                .iter()
                .copied()
                .filter(|column| *column != T::ID_COLUMN)
                .filter(|column| Some(*column) != T::VERSION_COLUMN)
                .filter(|column| Some(*column) != T::LOGIC_DELETE_COLUMN)
                .map(|column| format!("{column} = ?"))
                .collect::<Vec<_>>();
            if assignments.is_empty() && T::VERSION_COLUMN.is_none() {
                return Err(PlusError::InvalidArgument(
                    "update has no mutable columns".to_owned(),
                ));
            }
            let mut args = T::COLUMNS
                .iter()
                .copied()
                .filter(|column| *column != T::ID_COLUMN)
                .filter(|column| Some(*column) != T::VERSION_COLUMN)
                .filter(|column| Some(*column) != T::LOGIC_DELETE_COLUMN)
                .map(|column| field(&value, column))
                .collect::<PlusResult<Vec<_>>>()?;
            let mut set_clause = assignments.join(", ");
            if let Some(version) = T::VERSION_COLUMN {
                if !set_clause.is_empty() {
                    set_clause.push_str(", ");
                }
                write!(set_clause, "{version} = {version} + 1").expect("writing to String");
            }
            let mut sql = format!(
                "UPDATE {} SET {set_clause} WHERE {} = ?",
                T::TABLE_NAME,
                T::ID_COLUMN
            );
            args.push(id_value);
            if let Some(version) = T::VERSION_COLUMN {
                write!(sql, " AND {version} = ?").expect("writing to String");
                args.push(field(&value, version)?);
            }
            append_logic_filter::<T>(&mut sql);
            let rows_affected = self
                .execute_on(&self.rbatis, "BaseMapper.updateById", sql, args)
                .await?;
            if rows_affected == 0 {
                return Err(PlusError::Mapper(
                    "optimistic lock conflict or row not found".to_owned(),
                ));
            }
            self.select_by_id(id)
                .await?
                .ok_or_else(|| PlusError::Mapper("updated row cannot be reloaded".to_owned()))
        })
    }

    fn update(&self, update: UpdateWrapper<T>) -> BoxFuture<'_, PlusResult<u64>> {
        Box::pin(async move {
            if update.assignments.is_empty() {
                return Err(PlusError::InvalidArgument(
                    "update wrapper has no assignments".to_owned(),
                ));
            }
            if update.query.predicates().is_empty() {
                return Err(PlusError::InvalidArgument(
                    "update wrapper requires at least one predicate".to_owned(),
                ));
            }
            let mut assignments = Vec::with_capacity(update.assignments.len());
            let mut args = Vec::with_capacity(update.assignments.len());
            for (column, value) in update.assignments {
                validate_column::<T>(&column)?;
                if column == T::ID_COLUMN
                    || Some(column.as_str()) == T::VERSION_COLUMN
                    || Some(column.as_str()) == T::LOGIC_DELETE_COLUMN
                {
                    return Err(PlusError::InvalidArgument(format!(
                        "column `{column}` cannot be assigned by update wrapper"
                    )));
                }
                assignments.push(format!("{column} = ?"));
                args.push(rbs::to_value(value).map_err(mapper_error)?);
            }
            let (where_clause, where_args) = compile_where::<T>(&update.query)?;
            args.extend(where_args);
            let sql = format!(
                "UPDATE {} SET {}{where_clause}",
                T::TABLE_NAME,
                assignments.join(", ")
            );
            self.execute_on(&self.rbatis, "BaseMapper.update", sql, args)
                .await
        })
    }

    fn delete_by_id(&self, id: Id) -> BoxFuture<'_, PlusResult<bool>> {
        Box::pin(async move {
            let id = rbs::to_value(id).map_err(mapper_error)?;
            let (sql, args) = if let Some(column) = T::LOGIC_DELETE_COLUMN {
                (
                    format!(
                        "UPDATE {} SET {column} = 1 WHERE {} = ? AND {column} = 0",
                        T::TABLE_NAME,
                        T::ID_COLUMN
                    ),
                    vec![id],
                )
            } else {
                (
                    format!("DELETE FROM {} WHERE {} = ?", T::TABLE_NAME, T::ID_COLUMN),
                    vec![id],
                )
            };
            let rows_affected = self
                .execute_on(&self.rbatis, "BaseMapper.deleteById", sql, args)
                .await?;
            Ok(rows_affected > 0)
        })
    }

    fn insert_batch(&self, entities: Vec<T>) -> BoxFuture<'_, PlusResult<Vec<T>>> {
        Box::pin(async move {
            if entities.is_empty() {
                return Ok(Vec::new());
            }
            let transaction = self.rbatis.acquire_begin().await.map_err(mapper_error)?;
            for entity in &entities {
                if let Err(error) = self.insert_on(&transaction, entity).await {
                    let _ = transaction.rollback().await;
                    return Err(error);
                }
            }
            transaction.commit().await.map_err(mapper_error)?;
            Ok(entities)
        })
    }
}

fn validate_metadata<T: TableMetadata>() -> PlusResult<()> {
    for identifier in std::iter::once(T::TABLE_NAME)
        .chain(T::COLUMNS.iter().copied())
        .chain(std::iter::once(T::ID_COLUMN))
    {
        if identifier.is_empty()
            || !identifier
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return Err(PlusError::InvalidArgument(format!(
                "unsafe SQL identifier `{identifier}`"
            )));
        }
    }
    if !T::COLUMNS.contains(&T::ID_COLUMN) {
        return Err(PlusError::InvalidArgument(
            "id column is not present in columns".to_owned(),
        ));
    }
    for configured in [T::VERSION_COLUMN, T::LOGIC_DELETE_COLUMN]
        .into_iter()
        .flatten()
    {
        if !T::COLUMNS.contains(&configured) {
            return Err(PlusError::InvalidArgument(format!(
                "configured column `{configured}` is not present in columns"
            )));
        }
    }
    Ok(())
}

fn model_value<T: Serialize>(entity: &T) -> PlusResult<Value> {
    rbs::to_value(entity).map_err(mapper_error)
}

fn invocation_args(invocation: &SqlInvocation) -> PlusResult<Vec<Value>> {
    invocation
        .parameters
        .iter()
        .map(|value| rbs::to_value(value).map_err(mapper_error))
        .collect()
}

fn field(value: &Value, column: &str) -> PlusResult<Value> {
    match value {
        Value::Map(map) => map
            .0
            .get(&Value::String(column.to_owned()))
            .cloned()
            .ok_or_else(|| PlusError::InvalidArgument(format!("missing model field `{column}`"))),
        _ => Err(PlusError::InvalidArgument(
            "model must serialize as a map".to_owned(),
        )),
    }
}

fn compile_where<T: TableMetadata>(query: &QueryWrapper<T>) -> PlusResult<(String, Vec<Value>)> {
    let mut clauses = Vec::new();
    let mut args = Vec::new();
    if let Some(column) = T::LOGIC_DELETE_COLUMN {
        clauses.push(format!("{column} = 0"));
    }
    for predicate in query.predicates() {
        validate_column::<T>(&predicate.column)?;
        let column = &predicate.column;
        match predicate.operator {
            Operator::IsNull => clauses.push(format!("{column} IS NULL")),
            Operator::IsNotNull => clauses.push(format!("{column} IS NOT NULL")),
            Operator::In => {
                let JsonValue::Array(values) = &predicate.value else {
                    return Err(PlusError::InvalidArgument(
                        "IN predicate requires an array".to_owned(),
                    ));
                };
                if values.is_empty() {
                    clauses.push("1 = 0".to_owned());
                } else {
                    clauses.push(format!(
                        "{column} IN ({})",
                        vec!["?"; values.len()].join(", ")
                    ));
                    for value in values {
                        args.push(rbs::to_value(value).map_err(mapper_error)?);
                    }
                }
            }
            operator => {
                let sql_operator = match operator {
                    Operator::Eq => "=",
                    Operator::Ne => "<>",
                    Operator::Gt => ">",
                    Operator::Ge => ">=",
                    Operator::Lt => "<",
                    Operator::Le => "<=",
                    Operator::Like => "LIKE",
                    Operator::In | Operator::IsNull | Operator::IsNotNull => unreachable!(),
                };
                clauses.push(format!("{column} {sql_operator} ?"));
                args.push(rbs::to_value(&predicate.value).map_err(mapper_error)?);
            }
        }
    }
    if clauses.is_empty() {
        Ok((String::new(), args))
    } else {
        Ok((format!(" WHERE {}", clauses.join(" AND ")), args))
    }
}

fn append_order<T: TableMetadata>(sql: &mut String, query: &QueryWrapper<T>) -> PlusResult<()> {
    if query.sorts().is_empty() {
        return Ok(());
    }
    let sorts = query
        .sorts()
        .iter()
        .map(|sort| {
            validate_column::<T>(&sort.column)?;
            let direction = match sort.direction {
                SortDirection::Asc => "ASC",
                SortDirection::Desc => "DESC",
            };
            Ok(format!("{} {direction}", sort.column))
        })
        .collect::<PlusResult<Vec<_>>>()?;
    sql.push_str(" ORDER BY ");
    sql.push_str(&sorts.join(", "));
    Ok(())
}

fn validate_column<T: TableMetadata>(column: &str) -> PlusResult<()> {
    if T::COLUMNS.contains(&column) {
        Ok(())
    } else {
        Err(PlusError::InvalidArgument(format!(
            "unknown column `{column}`"
        )))
    }
}

fn append_logic_filter<T: TableMetadata>(sql: &mut String) {
    if let Some(column) = T::LOGIC_DELETE_COLUMN {
        write!(sql, " AND {column} = 0").expect("writing to String");
    }
}

fn mapper_error(error: impl std::fmt::Display) -> PlusError {
    PlusError::Mapper(error.to_string())
}
