use crate::{Accessor, ParsedCode, ParsedField};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens, quote_spanned};
use syn::*;

impl ParsedCode {
    pub fn into_tokens(self) -> TokenStream {
        let struct_ident = self.struct_ident;
        let accessor_field_impls = self
            .fields
            .into_iter()
            .map(|field| field.accessor_field_impl(struct_ident.clone()));

        quote! {
                #(#accessor_field_impls)*
        }
    }

		fn check_soundness(&self) -> TokenStream {
			let struct_ident = &self.struct_ident;
			let mut assertions = TokenStream::new();

			// asserts all invalidate(...) fields are actually present
			self.fields.iter().for_each(|f| {
				let field_name = f.name.clone();
				assertions.extend(quote_spanned!{field_name.span()=>
					::computed::static_assertions::assert_fields!(#struct_ident: #field_name);
				}.into_token_stream());
			});



			assertions
		}
}

impl ParsedField {
    pub fn accessor_field_impl(self, struct_ident: Ident) -> TokenStream {
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
