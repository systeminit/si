extern crate proc_macro;

mod dal_test;

use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, ItemFn};

#[proc_macro_attribute]
pub fn dal_test(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as AttributeArgs);
    let item = parse_macro_input!(input as ItemFn);
    dal_test::expand(item, args).into()
}
