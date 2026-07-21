use rbatis_plus::{PlusModel, TableMetadata};

#[derive(PlusModel)]
#[rbatis_plus(table_name = "orders", id_column = "order_id")]
struct OrderPo {
    order_id: i64,
    name: String,
}

#[test]
fn derive_emits_table_columns_and_id_metadata() {
    assert_eq!(OrderPo::TABLE_NAME, "orders");
    assert_eq!(OrderPo::ID_COLUMN, "order_id");
    assert_eq!(OrderPo::COLUMNS, &["order_id", "name"]);
    let row = OrderPo {
        order_id: 1,
        name: "created".into(),
    };
    assert_eq!(row.order_id, 1);
    assert_eq!(row.name, "created");
}
