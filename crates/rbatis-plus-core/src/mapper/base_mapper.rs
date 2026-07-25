// Source: mybatis-plus-core/.../mapper/BaseMapper.java

use async_trait::async_trait;
use rbs::Value;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::conditions::query::QueryWrapper;
use crate::conditions::update::UpdateWrapper;
use crate::page::Page;

/// The base mapper trait — provides CRUD operations.
///
/// Mirrors Java `com.baomidou.mybatisplus.core.mapper.BaseMapper<T>`.
///
/// Implementations typically use `rbatis::Executor` to execute SQL.
#[async_trait]
pub trait BaseMapper<T: Serialize + DeserializeOwned + Send + Sync>: Send + Sync {
    /// Insert an entity.
    ///
    /// 插入一条记录
    async fn insert(&self, entity: &T) -> Result<u64, rbatis::Error>;

    /// Delete by primary key.
    ///
    /// 根据 ID 删除
    async fn delete_by_id(&self, id: &Value) -> Result<u64, rbatis::Error>;

    /// Delete by QueryWrapper conditions.
    ///
    /// 根据条件删除
    async fn delete(&self, wrapper: &QueryWrapper, table_name: &str) -> Result<u64, rbatis::Error>;

    /// Update by primary key (full entity).
    ///
    /// 根据 ID 更新
    async fn update_by_id(&self, entity: &T) -> Result<u64, rbatis::Error>;

    /// Update by QueryWrapper conditions (SET from UpdateWrapper).
    ///
    /// 根据条件更新
    async fn update(
        &self,
        wrapper: &UpdateWrapper,
        table_name: &str,
    ) -> Result<u64, rbatis::Error>;

    /// Select by primary key.
    ///
    /// 根据 ID 查询
    async fn select_by_id(&self, id: &Value) -> Result<Option<T>, rbatis::Error>;

    /// Select by QueryWrapper conditions.
    ///
    /// 根据条件查询列表
    async fn select_list(
        &self,
        wrapper: &QueryWrapper,
        table_name: &str,
    ) -> Result<Vec<T>, rbatis::Error>;

    /// Select one by QueryWrapper conditions.
    ///
    /// 根据条件查询单条
    async fn select_one(
        &self,
        wrapper: &QueryWrapper,
        table_name: &str,
    ) -> Result<Option<T>, rbatis::Error>;

    /// Count by QueryWrapper conditions.
    ///
    /// 根据条件查询总数
    async fn select_count(
        &self,
        wrapper: &QueryWrapper,
        table_name: &str,
    ) -> Result<u64, rbatis::Error>;

    /// Paginated query.
    ///
    /// 分页查询
    async fn select_page(
        &self,
        wrapper: &QueryWrapper,
        table_name: &str,
        page_no: u64,
        page_size: u64,
    ) -> Result<Page<T>, rbatis::Error>;

    /// Check if any record matches the conditions.
    ///
    /// 根据条件判断是否存在
    async fn exists(
        &self,
        wrapper: &QueryWrapper,
        table_name: &str,
    ) -> Result<bool, rbatis::Error> {
        let count = self.select_count(wrapper, table_name).await?;
        Ok(count > 0)
    }
}
