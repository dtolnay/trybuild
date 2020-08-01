extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn warn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    eprintln!("Warning: You're using a silly proc_macro");
    item
}
