use std::collections::HashSet;

use proc_macro_error::abort;
use syn::*;

use crate::{Accessor, ParsedAttr, ParsedCode, ParsedField};

impl ParsedCode {
    pub fn new(fields: Vec<Field>, struct_ident: Ident) -> ParsedCode {
        let fields: Vec<ParsedField> = fields
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

        ParsedCode {
            struct_ident,
            fields,
        }
    }
}

impl ParsedAttr {
    pub fn from_attributes(attrs: Vec<Attribute>) -> ParsedAttr {
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
