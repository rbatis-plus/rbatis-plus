//! Procedural macros for RBatis-Plus.
//!
//! 核心宏：`#[derive(TableName)]` — 一次性生成所有元数据：
//! 表名、列访问器、主键、乐观锁、逻辑删除、字段填充。

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Meta, Fields};

/// `#[derive(TableName)]` — 表级元数据 + 列访问器 + 字段注解处理。
///
/// # struct 属性
/// - `#[table_name = "sys_user"]` — 表名
///
/// # 字段属性
/// - `#[table_id]` — 主键
/// - `#[table_id(type = "auto")]` — 主键生成策略（auto/input/assign_id/assign_uuid/none）
/// - `#[table_field(column = "db_name")]` — 自定义列名
/// - `#[version]` — 乐观锁版本列
/// - `#[table_logic]` — 逻辑删除列
/// - `#[table_logic(value = "1", not_value = "0")]` — 逻辑删除值
/// - `#[field_fill = "insert"]` — 自动填充（default/insert/update/insert_update）
/// - `#[encrypted(algorithm = "AES", column = "enc_name")]` — 加密字段
/// - `#[signature(order = 1, stored = true)]` — 签名字段
/// - `#[i18n(key = "user.name")]` — 国际化字段
#[proc_macro_derive(TableName, attributes(
    table_name, table_id, table_field, version, table_logic,
    field_fill, encrypted, signature, i18n
))]
pub fn derive_table_name(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // ── 提取 struct 级属性 ──
    let table_name = extract_string_attr(&input.attrs, "table_name")
        .unwrap_or_else(|| to_snake_case(&name.to_string()));

    // ── 扫描字段 ──
    let mut column_methods = Vec::new();
    let mut column_consts = Vec::new();

    // 主键信息
    let mut id_column = "id".to_string();
    let mut id_type_str = "none".to_string();

    // 乐观锁
    let mut has_version = false;
    let mut version_column = String::new();

    // 逻辑删除
    let mut has_logic = false;
    let mut logic_column = String::new();
    let mut logic_value = "0".to_string();
    let mut not_logic_value = "1".to_string();

    if let syn::Data::Struct(ref data_struct) = input.data {
        if let Fields::Named(ref fields) = data_struct.fields {
            for field in fields.named.iter() {
                let field_name = field.ident.as_ref().unwrap();
                let field_type = &field.ty;
                let db_column = extract_field_column(field);

                // 生成列访问器
                let method_name = syn::Ident::new(
                    &format!("column_{}", field_name), field_name.span(),
                );
                let const_name = syn::Ident::new(
                    &format!("COLUMN_{}", field_name.to_string().to_uppercase()),
                    field_name.span(),
                );

                column_methods.push(quote! {
                    pub fn #method_name() -> rbatis_plus_core::conditions::query::Column<#field_type> {
                        rbatis_plus_core::conditions::query::Column::new(#db_column)
                    }
                });
                column_consts.push(quote! {
                    pub const #const_name: &'static str = #db_column;
                });

                // ── 检查字段属性 ──

                // #[table_id] 或 #[table_id(type = "auto")]
                if has_attr(&field.attrs, "table_id") {
                    id_column = db_column.clone();
                    if let Some(t) = extract_nested_string(&field.attrs, "table_id", "type") {
                        id_type_str = t;
                    }
                }

                // #[version]
                if has_attr(&field.attrs, "version") {
                    has_version = true;
                    version_column = db_column.clone();
                }

                // #[table_logic] 或 #[table_logic(value = "1", not_value = "0")]
                if has_attr(&field.attrs, "table_logic") {
                    has_logic = true;
                    logic_column = db_column.clone();
                    if let Some(v) = extract_nested_string(&field.attrs, "table_logic", "value") {
                        logic_value = v;
                    }
                    if let Some(v) = extract_nested_string(&field.attrs, "table_logic", "not_value") {
                        not_logic_value = v;
                    }
                }
            }
        }
    }

    // ── 生成 IdType 枚举匹配 ──
    let id_type_expr = match id_type_str.to_lowercase().as_str() {
        "auto" => quote! { rbatis_plus_core::derive::IdType::Auto },
        "input" => quote! { rbatis_plus_core::derive::IdType::Input },
        "assign_id" => quote! { rbatis_plus_core::derive::IdType::AssignId },
        "assign_uuid" => quote! { rbatis_plus_core::derive::IdType::AssignUuid },
        _ => quote! { rbatis_plus_core::derive::IdType::None },
    };

    // ── 生成 TableName impl ──
    let table_name_impl = quote! {
        impl #impl_generics rbatis_plus_core::derive::TableName for #name #ty_generics #where_clause {
            fn table_name() -> &'static str { #table_name }
        }
    };

    // ── 生成 TableId impl ──
    let id_col_str = id_column.clone();
    let table_id_impl = quote! {
        impl #impl_generics rbatis_plus_core::derive::TableId for #name #ty_generics #where_clause {
            fn id_type() -> rbatis_plus_core::derive::IdType { #id_type_expr }
            fn id_column() -> &'static str { #id_col_str }
        }
    };

    // ── 生成 Version impl ──
    let version_impl = if has_version {
        let vc = version_column.clone();
        quote! {
            impl #impl_generics rbatis_plus_core::derive::Version for #name #ty_generics #where_clause {
                fn version_column() -> &'static str { #vc }
            }
        }
    } else {
        quote! {}
    };

    // ── 生成 TableLogic impl ──
    let logic_impl = if has_logic {
        let lc = logic_column.clone();
        let lv = logic_value.clone();
        let nlv = not_logic_value.clone();
        quote! {
            impl #impl_generics rbatis_plus_core::derive::TableLogic for #name #ty_generics #where_clause {
                fn logic_column() -> &'static str { #lc }
                fn logic_delete_value() -> Option<&'static str> {
                    if #lv.is_empty() { None } else { Some(#lv) }
                }
                fn logic_not_delete_value() -> Option<&'static str> {
                    if #nlv.is_empty() { None } else { Some(#nlv) }
                }
            }
        }
    } else {
        quote! {}
    };

    // ── 组装 ──
    let expanded = quote! {
        #table_name_impl
        #table_id_impl
        #version_impl
        #logic_impl

        impl #impl_generics #name #ty_generics #where_clause {
            #(#column_methods)*
            #(#column_consts)*
        }
    };
    expanded.into()
}

// ═══════════════════════════════════════════════════════════════════════════
// 辅助函数
// ═══════════════════════════════════════════════════════════════════════════

/// 提取 `#[attr_name = "value"]`。
fn extract_string_attr(attrs: &[syn::Attribute], attr_name: &str) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident(attr_name) {
            if let Meta::NameValue(mnv) = &attr.meta {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &mnv.value {
                    return Some(s.value());
                }
            }
        }
    }
    None
}

/// 检查字段是否有某个属性（`#[attr_name]` 或 `#[attr_name(...)]`）。
fn has_attr(attrs: &[syn::Attribute], attr_name: &str) -> bool {
    attrs.iter().any(|a| a.path().is_ident(attr_name))
}

/// 提取嵌套属性 `#[attr_name(key = "value")]`。
fn extract_nested_string(attrs: &[syn::Attribute], attr_name: &str, key: &str) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident(attr_name) {
            if let Meta::List(list) = &attr.meta {
                let mut result = None;
                let _ = list.parse_nested_meta(|meta| {
                    // 无论是否匹配 key，都需要消耗值以正确前进到下一个参数
                    if meta.path.is_ident(key) {
                        if let Ok(value) = meta.value() {
                            if let Ok(lit) = value.parse::<syn::LitStr>() {
                                result = Some(lit.value());
                            }
                        }
                    } else {
                        // 跳过不匹配的值（消耗掉以继续解析下一个参数）
                        let _ = meta.value().and_then(|v| v.parse::<syn::Expr>());
                    }
                    Ok(())
                });
                if result.is_some() {
                    return result;
                }
            }
        }
    }
    None
}

/// 从字段提取数据库列名（`#[table_field(column = "xxx")]` 或字段名原样）。
fn extract_field_column(field: &syn::Field) -> String {
    extract_nested_string(&field.attrs, "table_field", "column")
        .unwrap_or_else(|| field.ident.as_ref().unwrap().to_string())
}

/// PascalCase → snake_case。
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 { result.push('_'); }
            result.extend(c.to_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}
