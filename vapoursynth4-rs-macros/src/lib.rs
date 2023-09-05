use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn frame_done_callback(_attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("item: '{item}'");

    let input = parse_macro_input!(item as syn::ItemFn);

    quote! {
        #input
    }
    .into()
}
