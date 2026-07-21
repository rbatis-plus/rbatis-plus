use rbatis::RBatis;
use rbatis_plus_core::{
    BaseMapper, Column, IService, PageRequest, QueryWrapper, ServiceImpl, SortDirection,
    UpdateWrapper,
};
use rbatis_plus_extension::RbatisMapper;
use rbatis_plus_macros::PlusModel;
use rbdc_sqlite::SqliteDriver;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PlusModel)]
#[rbatis_plus(
    table_name = "orders",
    id_column = "id",
    version_column = "version",
    logic_delete_column = "deleted"
)]
struct OrderPo {
    id: i64,
    name: String,
    version: i64,
    deleted: i64,
}

async fn mapper() -> RbatisMapper<OrderPo, i64> {
    let rbatis = RBatis::new();
    rbatis
        .link(SqliteDriver {}, "sqlite://:memory:")
        .await
        .unwrap();
    rbatis
        .exec(
            "CREATE TABLE orders (id INTEGER PRIMARY KEY, name TEXT NOT NULL, \
             version INTEGER NOT NULL, deleted INTEGER NOT NULL)",
            vec![],
        )
        .await
        .unwrap();
    RbatisMapper::new(rbatis).unwrap()
}

fn order(id: i64, name: &str) -> OrderPo {
    OrderPo {
        id,
        name: name.to_owned(),
        version: 0,
        deleted: 0,
    }
}

#[tokio::test]
async fn executes_crud_query_page_optimistic_lock_and_logical_delete() {
    let mapper = Arc::new(mapper().await);
    let service = ServiceImpl::new(mapper.clone());
    service
        .save_batch(vec![order(1, "alpha"), order(2, "beta"), order(3, "beta")])
        .await
        .unwrap();

    let name = Column::<OrderPo>::new("name");
    let id = Column::<OrderPo>::new("id");
    let query = QueryWrapper::default()
        .eq(&name, "beta")
        .unwrap()
        .order_by(&id, SortDirection::Desc);
    let rows = service.list(query.clone()).await.unwrap();
    assert_eq!(rows.iter().map(|row| row.id).collect::<Vec<_>>(), [3, 2]);

    let page = service
        .page(PageRequest::new(1, 1).unwrap(), query)
        .await
        .unwrap();
    assert_eq!(page.total, 2);
    assert_eq!(page.records[0].id, 3);

    let affected = service
        .update(
            UpdateWrapper::default()
                .set(&name, "renamed")
                .unwrap()
                .where_query(QueryWrapper::default().r#in(&id, vec![2, 3]).unwrap()),
        )
        .await
        .unwrap();
    assert_eq!(affected, 2);
    let renamed = service
        .list(QueryWrapper::default().like(&name, "renamed").unwrap())
        .await
        .unwrap();
    assert_eq!(renamed.len(), 2);

    let updated = service
        .update_by_id(OrderPo {
            name: "paid".to_owned(),
            ..order(1, "alpha")
        })
        .await
        .unwrap();
    assert_eq!(updated.version, 1);
    assert_eq!(updated.name, "paid");

    let stale = service.update_by_id(order(1, "stale")).await.unwrap_err();
    assert!(stale.to_string().contains("optimistic lock conflict"));

    assert!(service.remove_by_id(1).await.unwrap());
    assert!(service.get_by_id(1).await.unwrap().is_none());
    let deleted: i64 = mapper
        .rbatis()
        .exec_decode("SELECT deleted FROM orders WHERE id = 1", vec![])
        .await
        .unwrap();
    assert_eq!(deleted, 1);
}

#[tokio::test]
async fn rejects_unbounded_or_protected_wrapper_updates() {
    let mapper = mapper().await;
    let name = Column::<OrderPo>::new("name");
    let id = Column::<OrderPo>::new("id");
    let unbounded = mapper
        .update(UpdateWrapper::default().set(&name, "unsafe").unwrap())
        .await
        .unwrap_err();
    assert!(unbounded.to_string().contains("at least one predicate"));

    let protected = mapper
        .update(
            UpdateWrapper::default()
                .set(&id, 2)
                .unwrap()
                .where_query(QueryWrapper::default().eq(&id, 1).unwrap()),
        )
        .await
        .unwrap_err();
    assert!(protected.to_string().contains("cannot be assigned"));
}

#[tokio::test]
async fn batch_insert_rolls_back_atomically_on_duplicate_key() {
    let mapper = mapper().await;
    mapper.insert(order(1, "existing")).await.unwrap();
    let error = mapper
        .insert_batch(vec![order(4, "rollback"), order(1, "duplicate")])
        .await
        .unwrap_err();
    assert!(error.to_string().contains("mapper error"));
    assert!(mapper.select_by_id(4).await.unwrap().is_none());
}

#[test]
fn rejects_untrusted_metadata_identifiers() {
    #[derive(Clone, Serialize, Deserialize)]
    struct UnsafeModel {
        id: i64,
    }
    impl rbatis_plus_core::TableMetadata for UnsafeModel {
        const TABLE_NAME: &'static str = "orders; DROP TABLE orders";
        const COLUMNS: &'static [&'static str] = &["id"];
        const ID_COLUMN: &'static str = "id";
    }
    let rbatis = RBatis::new();
    assert!(RbatisMapper::<UnsafeModel, i64>::new(rbatis).is_err());
}
