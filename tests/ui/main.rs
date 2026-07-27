// trybuild 主入口：自动扫描 tests/ui/*.rs 并验证编译结果。

#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/table_name_pass.rs");
    t.compile_fail("tests/ui/table_name_fail_empty_struct.rs");
}
