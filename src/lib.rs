// Read https://docs.rs/proc-macro-error/latest/proc_macro_error/#macros

use std::collections::HashSet;

use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote};
use syn::{
    parenthesized, parse_macro_input, Attribute, DataStruct, DeriveInput, Field, Fields,
    FieldsNamed, Meta, MetaList, Path, Type,
};

#[proc_macro_error]
#[proc_macro_derive(Computed, attributes(computed))]
pub fn computed(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    computed_impl(input).into()
}

mod parsing;
mod generating;

struct ParsedCode {
    struct_ident: Ident,
    fields: Vec<ParsedField>,
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

#[derive(Hash, Clone, PartialEq, Eq, Copy)]
enum Accessor {
    Get,
    Set,
}

fn computed_impl(input: DeriveInput) -> proc_macro2::TokenStream {
    match input.data {
        syn::Data::Struct(DataStruct { fields, .. }) => match fields {
            Fields::Named(FieldsNamed { named, .. }) => {
                let struct_ident = input.ident;
                let parsed: ParsedCode = ParsedCode::new(named.into_iter().collect(), struct_ident);

                parsed.into_tokens()
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
