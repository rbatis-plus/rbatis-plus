#![forbid(unsafe_code)]
//! Derive macros replacing Java lambda reflection with compile-time metadata.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr, parse_macro_input};

#[proc_macro_derive(PlusModel, attributes(rbatis_plus))]
pub fn derive_plus_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = input.ident;
    let mut table_name = name.to_string().to_lowercase();
    let mut id_column = "id".to_owned();
    for attribute in input
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("rbatis_plus"))
    {
        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("table_name") {
                table_name = meta.value()?.parse::<LitStr>()?.value();
            } else if meta.path.is_ident("id_column") {
                id_column = meta.value()?.parse::<LitStr>()?.value();
            } else {
                return Err(meta.error("supported keys: table_name, id_column"));
            }
            Ok(())
        })?;
    }
    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    name,
                    "PlusModel requires named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                name,
                "PlusModel supports structs only",
            ));
        }
    };
    let columns = fields
        .iter()
        .map(|field| field.ident.as_ref().expect("named").to_string())
        .collect::<Vec<_>>();
    if !columns.iter().any(|column| column == &id_column) {
        return Err(syn::Error::new_spanned(
            name,
            format!("id column `{id_column}` is not a field"),
        ));
    }
    Ok(quote! {
        impl ::rbatis_plus_core::TableMetadata for #name {
            const TABLE_NAME: &'static str = #table_name;
            const COLUMNS: &'static [&'static str] = &[#(#columns),*];
            const ID_COLUMN: &'static str = #id_column;
        }
    })
}
