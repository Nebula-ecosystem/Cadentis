mod args;
use args::MainArgs;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as MainArgs);
    let input = parse_macro_input!(item as ItemFn);

    let attrs = &input.attrs;
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;

    if sig.asyncness.is_none() {
        return Error::new_spanned(
            sig.fn_token,
            "#[executor::main] must be used on an async function",
        )
        .to_compile_error()
        .into();
    }

    if sig.ident != "main" {
        return Error::new_spanned(&sig.ident, "#[executor::main] must be used on fn main")
            .to_compile_error()
            .into();
    }

    let worker_threads = args.worker_threads;

    quote! {
        #(#attrs)*
        #vis fn main() {
            executor::Runtime::builder()
                .worker_threads(#worker_threads)
                .build()
                .expect("failed to build runtime")
                .block_on(async #block)
        }
    }
    .into()
}
