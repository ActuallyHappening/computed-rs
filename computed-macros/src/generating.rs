use std::collections::HashMap;

use crate::{Accessor, ParsedCode, ParsedField};
use derive_new::new;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::*;

#[derive(new)]
struct LinkedCode {
    struct_ident: Ident,

    #[new(default)]
    computed_fields: Vec<ComputedField>,

    #[new(default)]
    invalidating_fields: Vec<(Ident, LinkedAccessor)>,
}

struct ComputedField {
    self_name: Ident,
    self_type: Type,

    depending_fields: Vec<(Ident, Type)>,
    computing_func: Path,
}

#[derive(Default)]
struct LinkedAccessor {
    get: bool,
    set_invalidating: Option<Ident>,
}

impl ParsedCode {
    pub fn into_tokens(self) -> TokenStream {
        self.check_soundness();
        todo!()
    }

    fn check_soundness(self) -> LinkedCode {
        let struct_ident = self.struct_ident;

        let mut fields = HashMap::new();
        for field in self.fields {
            fields.insert(field.self_name.clone(), field);
        }

        let mut computed_fields = HashMap::new();

        // extract fields that are computed
        for field in fields.values() {
            match &field.attrs.computed {
                Some(computing_func) => {
                    let self_name = field.self_name.clone();
                    let self_type = field.self_type.clone();

                    if let Some(invalidates) = &field.attrs.invalidates {
                        abort!(
                            invalidates,
                            "Invalidates attribute is not allowed on a computed field"
                        );
                    }
                    if !field.attrs.accessors.is_empty() {
                        abort!(field.self_name, "Accessor attributes are not allowed on a computed field"; help = "You can access your computed field by calling the generated function `compute_foo()`");
                    }

                    computed_fields.insert(
                        self_name.clone(),
                        ComputedField {
                            self_name,
                            self_type,
                            computing_func: computing_func.clone(),
                            depending_fields: Vec::new(),
                        },
                    );
                }
                None => {}
            }
        }
        for computed_field in computed_fields.keys() {
            fields.remove(computed_field).unwrap();
        }

        let invalidating_fields = Vec::new();

        LinkedCode {
            struct_ident,
            computed_fields: computed_fields.into_values().into_iter().collect(),
            invalidating_fields,
        }
    }
}

impl ParsedField {
    pub fn accessor_field_impl(self, struct_ident: Ident) -> TokenStream {
        // accessors
        let attrs = &self.attrs;
        let accessor_fns = attrs.accessors.iter().map(|accessor| {
            (*accessor).accessor_fns(
                self.self_name.clone(),
                self.self_type.clone(),
                attrs.invalidates.clone(),
            )
        });

        quote! {
            impl #struct_ident {
                #(#accessor_fns)*
            }
        }
    }
}

impl Accessor {
    /// pub fn get_#field_ident(&self) -> &#type_name
    pub fn accessor_fns(
        self,
        field_ident: Ident,
        type_name: Type,
        invalidates: Option<Ident>,
    ) -> TokenStream {
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
                let invalidates = match invalidates {
                    Some(invalidates_field) => quote! { self.#invalidates_field.take() },
                    None => quote! {},
                };
                quote! {
                    pub fn #func_name(&mut self, #field_ident: #type_name) {
                                                #invalidates;
                        self.#field_ident = #field_ident;
                    }
                }
            }
        }
    }
}
