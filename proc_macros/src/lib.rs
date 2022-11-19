extern crate proc_macro;

use bevy_macro_utils::get_named_struct_fields;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{self, Parser},
    parse_macro_input, DeriveInput, ItemStruct,
};

enum Missing {
    Drop,
    Create,
    Ignore,
}

fn parse_attribute(attrs: &[syn::Attribute], name: &str, default: &str) -> String {
    use syn::spanned::Spanned;

    attrs
        .iter()
        .find(|a| a.path.is_ident(name))
        .map(|a| {
            a.tokens
                .clone()
                .into_iter()
                // Taking the second part of tokens, after the `=` sign.
                .nth(1)
                .ok_or_else(|| {
                    syn::Error::new(
                        a.span(),
                        format!(
                            r#"The attribute should be in the format: `{} = "{}"`"#,
                            name, default
                        ),
                    )
                })
                .unwrap()
                .to_string()
                .trim_matches('\"')
                .to_owned()
        })
        .unwrap_or_else(|| default.to_string())
}

// using proc_macro_attribute to declare an attribute like procedural macro
#[proc_macro_derive(NetworkEvent, attributes(entity, missing))]
// _metadata is argument provided to macro call and _input is code to which attribute like macro attaches
pub fn derive_network_event(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let named_fields = match get_named_struct_fields(&ast.data) {
        Ok(fields) => &fields.named,
        Err(e) => return e.into_compile_error().into(),
    };

    let mut drop_network_to_entity = Vec::<TokenStream>::new();
    let mut drop_entity_to_network = Vec::<TokenStream>::new();
    let mut create_network_to_entity = Vec::<TokenStream>::new();
    let mut create_entity_to_network = Vec::<TokenStream>::new();
    let mut ignore_network_to_entity = Vec::<TokenStream>::new();
    let mut ignore_entity_to_network = Vec::<TokenStream>::new();

    let network_entity_fields = named_fields
        .into_iter()
        .filter(|field| {
            field
                .attrs
                .iter()
                .any(|attr| *attr.path.get_ident().unwrap() == "entity")
        })
        .map(|field| {
            let missing = match parse_attribute(&field.attrs, "missing", "drop")
                .to_lowercase()
                .as_str()
            {
                "drop" => Missing::Drop,
                "create" => Missing::Create,
                "ignore" => Missing::Ignore,
                _ => panic!("missing has invalid value"),
            };
            let ident = field.ident.clone();
            (ident, missing)
        })
        .collect::<Vec<_>>();

    for field in network_entity_fields {
        match field.1 {
            Missing::Drop => {
                let ident = field.0;
                drop_network_to_entity.push(
                    quote! {
                        if let Some(entity) = network_id_map.from_network(self.#ident.into()) {
                            self.#ident = entity;
                        } else {
                            return false;
                        }
                    }
                    .into(),
                );

                drop_entity_to_network.push(
                    quote! {
                        if let Some(network_id) = network_id_map.from_entity(self.#ident.into()) {
                            self.#ident = network_id.into();
                        } else {
                            return false;
                        }
                    }
                    .into(),
                );
            }
            Missing::Create => {
                let ident = field.0;
                create_network_to_entity.push(
                    quote! {
                        if let Some(entity) = network_id_map.from_network(self.#ident.into()) {
                            self.#ident = entity;
                        } else {
                            let entity = commands.spawn().id();
                            network_id_map.insert_with_id(entity, self.#ident.into());
                            self.#ident = entity;
                        }
                    }
                    .into(),
                );

                create_entity_to_network.push(
                    quote! {
                        if let Some(network_id) = network_id_map.from_entity(self.#ident.into()) {
                            self.#ident = network_id.into();
                        } else {
                            self.#ident = network_id_map.insert(self.#ident).into();
                        }
                    }
                    .into(),
                );
            }
            Missing::Ignore => {
                let ident = field.0;

                ignore_network_to_entity.push(
                    quote! {
                        if let Some(network_id) = self.#ident {
                            self.#ident = network_id_map.from_network(network_id);
                        }
                    }
                    .into(),
                );

                ignore_entity_to_network.push(
                    quote! {
                        if let Some(entity) = self.#ident {
                            self.#ident = network_id_map.from_entity(entity);
                        }
                    }
                    .into(),
                );
            }
        }
    }

    let ident = ast.ident;

    proc_macro::TokenStream::from(quote! {
        impl spacegame_core::NetworkEvent for #ident {

            fn entity_to_network(&mut self, network_id_map: &mut spacegame_core::network_id::NetworkIdMap) -> bool {
               #(#drop_entity_to_network)*
               #(#create_entity_to_network)*
               #(#ignore_entity_to_network)*
               return true;
            }

            fn network_to_entity(&mut self, commands: &mut bevy::prelude::Commands, network_id_map: &mut spacegame_core::network_id::NetworkIdMap) -> bool {
                #(#drop_network_to_entity)*
                #(#create_network_to_entity)*
                #(#ignore_network_to_entity)*
                return true;
            }
        }

        impl spacegame_core::message::NetworkEventChannelId for #ident {
            const CHANNEL_ID: u8 = 0;
        }
    })
}

#[proc_macro_attribute]
pub fn server_bound(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);
    let _ = parse_macro_input!(args as parse::Nothing);

    if let syn::Fields::Named(ref mut fields) = item_struct.fields {
        fields.named.push(
            syn::Field::parse_named
                .parse2(quote! {
                    pub sender_id: spacegame_core::message::ClientId
                })
                .unwrap(),
        );
    }

    return quote! {
        #[derive(unique_type_id_derive::UniqueTypeId, spacegame_proc_macros::NetworkEvent)]
        #[UniqueTypeIdType = "u16"]
        #item_struct
    }
    .into();
}

#[proc_macro_attribute]
pub fn client_bound(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);
    let _ = parse_macro_input!(args as parse::Nothing);

    return quote! {
        #[derive(unique_type_id_derive::UniqueTypeId, spacegame_proc_macros::NetworkEvent)]
        #[UniqueTypeIdType = "u16"]
        #item_struct
    }
    .into();
}
