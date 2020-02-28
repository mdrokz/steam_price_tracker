extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;

use postgres_types::ToSql;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use std::vec::Vec;
use syn::parse;
use syn::Data;

#[proc_macro_derive(ExtractStructs)]
pub fn macro_web_derive(input: TokenStream) -> TokenStream {
    // Construct a represntation of Rust code as a syntax tree
    // that we can manipulate
    let ast = parse(input).unwrap();

    // Build the trait implementation
    impl_web_macro(&ast)
}

fn impl_web_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let body = &ast.data;
    let mut n_name: Vec<Option<&syn::Ident>> = vec![];
    let sdata: Option<&syn::DataStruct> = match body {
        Data::Struct(data) => Some(data),
        Data::Enum(_data) => None,
        Data::Union(_data) => None,
    };
    let d = sdata.unwrap();
    for (i, field) in d.fields.iter().enumerate() {
        let val = match &field.ident {
            Some(data) => Some(data),
            None => None,
        };
        n_name.push(val);
    }
    let gen = quote! {
        impl ExtractStructs for #name {
            fn extract(data:&Self) -> Vec<&(dyn ToSql + Sync)>  {
                let mut vecc:Vec<&(dyn ToSql + Sync)> = vec![];
                [#(vecc.push(&data.#n_name)),*];
               vecc
            }

            fn map_pg_values(&mut self,pg_row:&Vec<Row>) {
                let mut i = 0;
                for row in pg_row {
                    #(self.#n_name = row.get(i); i = i + 1);*;
                }
            }

        }
    };
    gen.into()
}
