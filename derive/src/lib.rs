use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(SerializeBytes)]
pub fn derive_serialize_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let (to_slice_impl, from_slice_impl, size_calc) = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => {
                let to_slice_fields = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    quote! {
                        w.write(&self.#name);
                    }
                });

                let from_slice_fields = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    quote! {
                        #name: r.read()?,
                    }
                });

                let size_fields = fields.named.iter().map(|f| {
                    let ty = &f.ty;
                    quote! {
                        #ty::SIZE +
                    }
                });

                (
                    quote! {
                        let mut w = Writer::new(out);
                        #(#to_slice_fields)*
                    },
                    quote! {
                        let mut r = Reader::new(input);
                        Ok(Self {
                            #(#from_slice_fields)*
                        })
                    },
                    quote! {
                        #(#size_fields)* 0
                    },
                )
            }
            _ => panic!("SerializeBytes can only be derived for structs with named fields"),
        },
        _ => panic!("SerializeBytes can only be derived for structs"),
    };

    let expanded = quote! {
        impl SerializeBytes for #name {
            const SIZE: usize = #size_calc;

            #[inline]
            fn to_slice(&self, out: &mut [u8]) {
                #to_slice_impl
            }

            #[inline]
            fn from_slice(input: &[u8]) -> Result<Self> {
                #from_slice_impl
            }
        }
    };

    TokenStream::from(expanded)
}
