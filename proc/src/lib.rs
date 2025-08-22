extern crate proc_macro;
use proc_macro::TokenStream;

use syn::{parse_macro_input, DataEnum, DataStruct, DeriveInput, Fields, FieldsUnnamed, Variant};

#[proc_macro_derive(Parseable)]
pub fn derive_answer_fn(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let DeriveInput { data, ident, attrs, generics, .. } = input;

    match data {
        syn::Data::Struct(DataStruct { fields, .. }) => {
            let fields = fields.into_iter().map(|field| {
                field.ident
            }).collect::<Vec<_>>();

            TokenStream::from(quote::quote! {
                #(#attrs)*
                impl #generics crate::parse::Parseable for #ident #generics {
                    fn parse(t: &mut crate::tokenize::TokenStream) -> Result<Self, crate::parse::ParsingError> where Self: Sized {
                        let mut cloned_t = t.clone();
                        #(let #fields = match <_>::parse(t) {
                            Ok(v) => v,
                            Err(e) => {
                                *t = cloned_t;
                                return Err(e);
                            }
                        };)*
                        Ok(Self {
                            #(#fields),*
                        })
                    }
                }
            })
        },
        syn::Data::Enum(DataEnum { variants, .. }) => {
            let mut var_attrs = vec![];
            let mut var_tys = vec![];
            let mut var_idents = vec![];

            for Variant { attrs, fields, ident, .. } in variants {
                let Fields::Unnamed(FieldsUnnamed { mut unnamed, .. }) = fields else { panic!("must have exactly one unnamed field") };
                let field = unnamed.pop().expect("must have exactly one unnamed field").into_value();
                assert!(unnamed.is_empty(), "must have exactly one unnamed field");
                var_idents.push(ident);
                var_tys.push(field.ty);
                var_attrs.push(attrs);
            }

            TokenStream::from(quote::quote! {
                #(#attrs)*
               impl #generics crate::parse::Parseable for #ident #generics {
                    fn parse(t: &mut crate::tokenize::TokenStream) -> Result<Self, crate::parse::ParsingError> where Self: Sized {
                        #( #(#var_attrs)* {
                            let cloned_t = t.clone();
                            if let Ok(s) = <#var_tys>::parse(t) {
                                return Ok(Self::#var_idents(s))
                            } else {
                                *t = cloned_t;
                            }
                        })*

                        return Err(crate::parse::ParsingError::new(t.char_index(), concat!("expected ", stringify!(#ident))))
                    }
                }
            })
        },
        _ => unimplemented!()
    }
}
