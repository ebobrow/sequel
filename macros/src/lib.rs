use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Data, DataEnum, DeriveInput, Lit, LitByteStr, Meta,
    MetaList, MetaNameValue, NestedMeta, Path,
};

#[proc_macro_derive(Keywords, attributes(keyword))]
pub fn derive_keywords(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let fields = if let Data::Enum(DataEnum { variants, .. }) = &input.data {
        variants
    } else {
        panic!("Must be a struct");
    };
    let keywords: Vec<_> = fields
        .iter()
        .filter(|field| field.attrs.len() == 1)
        .map(|field| {
            match field.attrs[0].parse_meta() {
                Ok(Meta::Path(_)) => {
                    // TODO: check path? (but don't need to for just me)
                    // TODO: this without manual `Lit` stuff?
                    let left = Lit::ByteStr(LitByteStr::new(
                        field.ident.to_string().to_uppercase().as_bytes(),
                        field.span(),
                    ));
                    let a = &input.ident;
                    let b = &field.ident;
                    quote! {
                        #left => Some(#a::#b),
                    }
                }
                Ok(Meta::List(MetaList { nested, .. })) => {
                    let mut lines = Vec::new();
                    for meta in nested {
                        if let NestedMeta::Meta(meta) = meta {
                            if let Meta::NameValue(MetaNameValue {
                                path: Path { segments, .. },
                                lit,
                                ..
                            }) = meta
                            {
                                let left = Lit::ByteStr(LitByteStr::new(
                                    format!("{}", segments[0].ident).as_bytes(),
                                    segments[0].span(),
                                ));
                                let a = &input.ident;
                                let b = &field.ident;
                                lines.push(quote! { #left => Some(#a::#b(#lit)), });
                            }
                        }
                    }
                    quote! { #(#lines)* }
                }
                _ => panic!("uh oh"),
            }
        })
        .collect();

    TokenStream::from(quote! {
        /// Helper struct with method `get` to search keywords
        pub struct Keyword;
        impl Keyword {
            /// Searches keyword for bytestring match, returns `Option<Token>`
            pub fn get(key: &[u8]) -> Option<Token> {
                match key {
                    #(#keywords)*
                    _ => None,
                }
            }
        }
    })
}
