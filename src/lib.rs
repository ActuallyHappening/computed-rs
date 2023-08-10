// Read https://docs.rs/proc-macro-error/latest/proc_macro_error/#macros

use std::collections::HashSet;

use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, emit_error, proc_macro_error};
use quote::{quote, ToTokens, format_ident};
use syn::{
    parenthesized, parse_macro_input, parse_quote, Attribute, DataStruct, DeriveInput, Fields,
    FieldsNamed, Meta, MetaList, Path, Type, TypeGroup,
};

#[proc_macro_error]
#[proc_macro_derive(Computed, attributes(computed))]
pub fn computed(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    computed_impl(input).into()
}

/// #[get, set, invalidates(sum)]
/// sum: Vec<f32>
struct ParsedField {
    /// Name of field, e.g. sum
    name: Ident,
    /// Name of type, e.g. Vec<f32>
    type_name: Type,
    /// Attributes of field, e.g. get, set, invalidates
    attrs: ParsedAttr,
}

/// get, set, invalidates, computed
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
    let struct_ident = &input.ident;
    match input.data {
        syn::Data::Struct(DataStruct { fields, .. }) => match fields {
            Fields::Named(FieldsNamed { named, .. }) => {
                let field_accessor_impls = named
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
                    .map(|field| field.into_tokens(struct_ident.clone()))
                    .collect::<Vec<_>>();

                quote! {
                    #(#field_accessor_impls)*
                }
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
            if let Attribute {
                meta: Meta::List(MetaList { path, .. }),
                ..
            } = &attr
            {
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

impl ParsedField {
    fn into_tokens(self, struct_ident: Ident) -> TokenStream {
        // validate

        // accessors
        let accessor_fns = self.attrs.accessors.iter().map(|accessor| {
            accessor
                .clone()
                .into_tokens(self.name.clone(), self.type_name.clone())
        });

        quote! {
            impl #struct_ident {
                #(#accessor_fns)*
            }
        }
    }
}

impl Accessor {
    fn into_tokens(self, field_ident: Ident, type_name: Type) -> TokenStream {
        match self {
            Accessor::Get => {
                let func_name = format_ident!("get_{}", field_ident);
                quote! {
                    pub fn #func_name(&self) -> &#type_name {
                        &self.#field_ident
                    }
                }
            }
            Accessor::Set => {
                let func_name = format_ident!("set_{}", field_ident);
                quote! {
                    pub fn #func_name(&mut self, #field_ident: #type_name) {
                        self.#field_ident = #field_ident;
                    }
                }
            }
        }
    }
}
