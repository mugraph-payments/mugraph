use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn timed(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let name = &input.sig.ident;
    let body = &input.block;
    let vis = &input.vis;
    let attrs = &input.attrs;
    let sig = &input.sig;

    let module_path = quote! { module_path!() };

    let output = quote! {
        #(#attrs)*
        #vis #sig {{
            use std::time::Instant;
            use mugraph_core::metrics::Metric;

            let start = ::std::time::Instant::now();
            let result = #body;

            Metric::increment(concat!(#module_path, "::", stringify!(#name)), start.elapsed());

            result
        }}
    };

    output.into()
}
