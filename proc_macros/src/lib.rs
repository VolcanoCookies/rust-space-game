extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse, Field, Item, ItemFn, ItemStruct};

// using proc_macro_attribute to declare an attribute like procedural macro
#[proc_macro_derive(NetworkEvent, attributes(networkEntity))]
// _metadata is argument provided to macro call and _input is code to which attribute like macro attaches
pub fn derive_network_event(input: TokenStream) -> TokenStream {
    let ItemStruct {
        attrs,
        vis,
        struct_token,
        ident,
        generics,
        fields,
        semi_token,
    } = parse(input).unwrap();

    let entity_fields = fields
        .iter()
        .filter(|field| {
            field
                .attrs
                .iter()
                .any(|attr| attr.path.get_ident().unwrap() == "networkEntity")
        })
        .collect::<Vec<_>>();

    let field_stream = fields
        .into_iter()
        .map(|field| {
            let vis = field.vis;
            let ident = field.ident;
            let ty = field.ty;
            quote! {
                #vis fn #ident(&self) -> #ty {
                    self.#ident
                }
            }
            .into_token_stream()
        })
        .collect::<Vec<_>>();
    // let mut entity_field_stream = Vec::new();

    quote! {
        impl #ident {
            #(#field_stream)*
        }
    }
    .into()
}
