// Read https://docs.rs/proc-macro-error/latest/proc_macro_error/#macros

use std::collections::HashSet;

use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, emit_error, proc_macro_error};
use quote::{quote, ToTokens};
use syn::{
    parenthesized, parse_macro_input, parse_quote, Attribute, DataStruct, DeriveInput, Fields,
    FieldsNamed, Path, Type, TypeGroup, Meta, MetaList,
};

#[proc_macro_error]
#[proc_macro_derive(Computed, attributes(computed))]
pub fn computed(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    computed_impl(input).into()
}

// struct ParsedCode {}

// struct GetAccessor {
//     field_ident: Ident,
//     type_name: TypeGroup,
// }
// struct SetAccessor {
//     field_ident: Ident,
//     type_name: Ident,
//     invalidates: Vec<Ident>,
// }

struct ParsedField {
    name: Ident,
    type_name: Type,
    attrs: ParsedAttr,
}

struct ParsedAttr {
    accessors: Vec<Accessor>,
    invalidates: Option<Ident>,
    computed: Option<Path>,
}

#[derive(Hash, Clone, PartialEq, Eq)]
enum Accessor {
    Get,
    Set,
}

fn computed_impl(input: DeriveInput) -> proc_macro2::TokenStream {
    match input.data {
        syn::Data::Struct(DataStruct { fields, .. }) => match fields {
            Fields::Named(FieldsNamed { named, .. }) => {
                let fields = named
                    .into_iter()
                    .map(|field| {
                        let name = field.ident.unwrap();
                        let type_name = field.ty;
                        let attrs = ParsedAttr::from_attributes(field.attrs);

                        ParsedField {
                            name,
                            type_name,
                            attrs,
                        }
                    })
                    .collect::<Vec<_>>();

                todo!()
            }
            _ => abort!(input.ident, "Only named fields are supported"),
        },
        syn::Data::Enum(syn::DataEnum {
            enum_token: token, ..
        }) => {
            abort!(token, "Only structs are supported")
        }
        syn::Data::Union(syn::DataUnion {
            union_token: token, ..
        }) => {
            abort!(token, "Only structs are supported")
        }
    }
}

impl ParsedAttr {
    fn from_attributes(attrs: Vec<Attribute>) -> ParsedAttr {
        let mut accessors = HashSet::new();
        let mut invalidates = None;
        let mut computed = None;

        for attr in attrs {
						if let Attribute { meta: Meta::List(MetaList { path, ..}), .. } = &attr {
							if !path.is_ident("computed") {
								continue;
							}
						}
            if let Err(err) = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("get") {
                    accessors.insert(Accessor::Get);
                } else if meta.path.is_ident("set") {
                    accessors.insert(Accessor::Set);
                } else if meta.path.is_ident("invalidates") {
                    let content;
                    parenthesized!(content in meta.input);

                    if invalidates.is_some() {
                        abort!(meta.path, "Already set invalidates for this field")
                    }
                    let field_ident = content.parse::<Ident>()?;
                    invalidates = Some(field_ident);
                } else if meta.path.is_ident("computed") {
                    let content;
                    parenthesized!(content in meta.input);

                    if computed.is_some() {
                        abort!(meta.path, "Already set computed for this field")
                    }
                    let path = content.parse::<Path>()?;
                    computed = Some(path);
                } else {
                    abort!(meta.path, "Unknown attribute")
                }
                Ok(())
            }) {
                abort!(attr, err)
            }
        }

        ParsedAttr {
            accessors: accessors.into_iter().collect(),
            invalidates,
            computed,
        }
    }
}

// impl ToTokens for GetAccessor {
//     fn to_tokens(&self, tokens: &mut TokenStream) {
//         let field_ident = &self.field_ident;
//         let type_name = &self.type_name;
//         let func_name = Ident::new(&format!("get_{}", field_ident), field_ident.span());
//         tokens.extend(quote! {
//             pub fn #func_name(&self) -> &#type_name {
//                 &self.#field_ident
//             }
//         });
//     }
// }
