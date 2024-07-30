use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(SerializeBytes)]
pub fn derive_serialize_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let (impl_block, size_calc) = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => {
                let field_data: Vec<_> = fields
                    .named
                    .iter()
                    .map(|f| {
                        let name = &f.ident;
                        let ty = &f.ty;
                        (name, ty)
                    })
                    .collect();

                let mut offset = quote! { 0usize };
                let to_slice_impl = field_data.iter().map(|(name, ty)| {
                    let size = quote! { <#ty as SerializeBytes>::SIZE };
                    let current = offset.clone();
                    offset = quote! { #offset + #size };

                    quote! {
                        <#ty as SerializeBytes>::to_slice(&self.#name, &mut out[#current..#offset]);
                    }
                });

                let mut offset = quote! { 0usize };
                let from_slice_impl = field_data.iter().map(|(name, ty)| {
                    let size = quote! { <#ty as SerializeBytes>::SIZE };
                    let current = offset.clone();
                    offset = quote! { #offset + #size };

                    quote! {
                        #name: <#ty as SerializeBytes>::from_slice(&input[#current..#offset])?,
                    }
                });

                let size_calc = field_data
                    .iter()
                    .map(|(_, ty)| {
                        quote! { <#ty as SerializeBytes>::SIZE }
                    })
                    .fold(quote!(0), |acc, size| {
                        quote! { #acc + #size }
                    });

                (
                    quote! {
                        fn to_slice(&self, out: &mut [u8]) {
                            #(#to_slice_impl)*
                        }

                        fn from_slice(input: &[u8]) -> Result<Self> {
                            Ok(Self {
                                #(#from_slice_impl)*
                            })
                        }
                    },
                    size_calc,
                )
            }
            _ => panic!("SerializeBytes can only be derived for structs with named fields"),
        },
        _ => panic!("SerializeBytes can only be derived for structs"),
    };

    let expanded = quote! {
        impl SerializeBytes for #name {
            const SIZE: usize = #size_calc;

            #impl_block
        }
    };

    TokenStream::from(expanded)
}
