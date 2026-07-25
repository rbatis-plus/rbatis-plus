// Source: mybatis-plus-extension/.../repository/IRepository.java
// Source: mybatis-plus-spring/.../service/IService.java

use async_trait::async_trait;
use rbatis::Error;
use rbatis_plus_core::conditions::query::QueryWrapper;
use rbatis_plus_core::conditions::update::UpdateWrapper;
use rbatis_plus_core::page::Page;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// The service-layer trait — provides high-level CRUD + batch operations.
///
/// Mirrors Java `IService<T>` (which extends `IRepository<T>`).
#[async_trait]
pub trait IService<T: Serialize + DeserializeOwned + Send + Sync>: Send + Sync {
    /// Insert one entity.
    ///
    /// 插入一条记录
    async fn save(&self, entity: &T) -> Result<bool, Error>;

    /// Insert or update by id.
    ///
    /// 插入或更新（根据 ID 判断）
    async fn save_or_update(&self, entity: &T) -> Result<bool, Error>;

    /// Remove by id.
    ///
    /// 根据 ID 删除
    async fn remove_by_id(&self, id: &rbs::Value) -> Result<bool, Error>;

    /// Remove by QueryWrapper.
    ///
    /// 根据条件删除
    async fn remove(&self, wrapper: &QueryWrapper, table_name: &str) -> Result<bool, Error>;

    /// Update by id (full entity).
    ///
    /// 根据 ID 更新
    async fn update_by_id(&self, entity: &T) -> Result<bool, Error>;

    /// Update by UpdateWrapper.
    ///
    /// 根据条件更新
    async fn update(
        &self,
        wrapper: &UpdateWrapper,
        table_name: &str,
    ) -> Result<bool, Error>;

    /// Get by id.
    ///
    /// 根据 ID 查询
    async fn get_by_id(&self, id: &rbs::Value) -> Result<Option<T>, Error>;

    /// List by QueryWrapper.
    ///
    /// 根据条件查询列表
    async fn list(&self, wrapper: &QueryWrapper, table_name: &str) -> Result<Vec<T>, Error>;

    /// Get one by QueryWrapper.
    ///
    /// 根据条件查询单条
    async fn get_one(
        &self,
        wrapper: &QueryWrapper,
        table_name: &str,
    ) -> Result<Option<T>, Error>;

    /// Count.
    ///
    /// 查询总数
    async fn count(&self, wrapper: &QueryWrapper, table_name: &str) -> Result<u64, Error>;

    /// Paginated query.
    ///
    /// 分页查询
    async fn page(
        &self,
        wrapper: &QueryWrapper,
        table_name: &str,
        page_no: u64,
        page_size: u64,
    ) -> Result<Page<T>, Error>;
}
