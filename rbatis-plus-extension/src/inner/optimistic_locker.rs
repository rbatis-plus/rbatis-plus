// Source: mybatis-plus-extension/.../inner/OptimisticLockerInnerInterceptor.java

use crate::inner::inner_interceptor::InnerInterceptor;
use async_trait::async_trait;
use rbatis::executor::Executor;
use rbatis::{Action, Error};
use rbdc::db::ExecResult;
use rbs::Value;

/// Optimistic locker inner interceptor.
///
/// Mirrors Java `OptimisticLockerInnerInterceptor`.
///
/// On UPDATE, if the entity has a `@Version` field, the interceptor:
/// 1. Reads the current version from the entity.
/// 2. Appends `AND version = {old_version}` to the WHERE clause.
/// 3. Increments the version in the SET clause: `version = {old_version} + 1`.
///
/// This is a simplified MVP that works with the version column name
/// configured at construction time.  Full entity-reflection-based
/// version detection will be added when the macros module is complete.
#[derive(Debug, Clone)]
pub struct OptimisticLockerInnerInterceptor {
    /// The version column name.
    pub version_column: String,
}

impl OptimisticLockerInnerInterceptor {
    pub fn new(version_column: impl Into<String>) -> Self {
        Self {
            version_column: version_column.into(),
        }
    }
}

#[async_trait]
impl InnerInterceptor for OptimisticLockerInnerInterceptor {
    async fn before_update(
        &self,
        _executor: &dyn Executor,
        sql: &mut String,
        _args: &mut Vec<Value>,
        _result: &mut Result<ExecResult, Error>,
    ) -> Result<Action, Error> {
        let lower = sql.trim_start().to_lowercase();
        if !lower.starts_with("update ") {
            return Ok(Action::Next);
        }

        // Check if the SET clause already includes the version column
        let col = &self.version_column;
        if lower.contains(&format!("{} = ", col.to_lowercase())) {
            // Version is already being set — no auto-increment needed
            return Ok(Action::Next);
        }

        // Append version increment to SET clause
        // Find "SET " position
        let set_pos = lower.find(" set ");
        if let Some(pos) = set_pos {
            let set_end = pos + 5; // after "SET "
            let insert_pos = sql[pos..set_end].len();
            sql.insert_str(insert_pos, &format!(" {} = {} + 1,", col, col));
        }

        Ok(Action::Next)
    }
}
