extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use quote::quote;

#[proc_macro_derive(Print)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();
    println!("IN PROC-MACRO");

    proc_macro::TokenStream::from(quote! {
        impl #name {}
    })
}
