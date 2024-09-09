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
        #vis #sig {
            let start = ::std::time::Instant::now();
            let result = #body;
            let duration = start.elapsed().as_nanos() as f64;
            let full_path = concat!(#module_path, "::", stringify!(#name));

            ::metrics::histogram!("mugraph.task.durations", "name" => full_path).record(duration);
            ::metrics::counter!("mugraph.task.calls", "name" => full_path).increment(1);

            result
        }
    };

    output.into()
}
