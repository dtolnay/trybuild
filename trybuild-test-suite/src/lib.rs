extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro]
pub fn test_stdout(_: TokenStream) -> TokenStream {
    print!("foo");

    "struct __F;".parse().unwrap()
}
