use rbatis_plus::{PlusModel, TableMetadata};

#[derive(PlusModel)]
#[rbatis_plus(
    table_name = "orders",
    id_column = "order_id",
    version_column = "version",
    logic_delete_column = "deleted",
    crate_path = "rbatis_plus"
)]
struct OrderPo {
    order_id: i64,
    name: String,
    version: i64,
    deleted: bool,
}

#[test]
fn derive_emits_table_columns_and_id_metadata() {
    assert_eq!(OrderPo::TABLE_NAME, "orders");
    assert_eq!(OrderPo::ID_COLUMN, "order_id");
    assert_eq!(
        OrderPo::COLUMNS,
        &["order_id", "name", "version", "deleted"]
    );
    assert_eq!(OrderPo::VERSION_COLUMN, Some("version"));
    assert_eq!(OrderPo::LOGIC_DELETE_COLUMN, Some("deleted"));
    let row = OrderPo {
        order_id: 1,
        name: "created".into(),
        version: 0,
        deleted: false,
    };
    assert_eq!(row.order_id, 1);
    assert_eq!(row.name, "created");
    assert_eq!(row.version, 0);
    assert!(!row.deleted);
}
