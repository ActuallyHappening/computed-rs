use crate::{ParsedField, Accessor, ParsedCode};
use proc_macro2::TokenStream;
use syn::*;
use quote::{quote, format_ident};

impl ParsedCode {
	 pub fn into_tokens(self) -> TokenStream {
				let struct_ident = self.struct_ident;
				let accessor_field_impls = self.fields.into_iter().map(|field| {
						field.accessor_field_impls(struct_ident.clone())
				});

				quote! {
						#(#accessor_field_impls)*
				}
    }
}

impl ParsedField {
    pub fn accessor_field_impls(self, struct_ident: Ident) -> TokenStream {
        // accessors
        let attrs = &self.attrs;
        let accessor_fns = attrs.accessors.iter().map(|accessor| {
            (*accessor).accessor_fns(
                self.name.clone(),
                self.type_name.clone(),
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