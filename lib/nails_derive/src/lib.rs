extern crate proc_macro;

use proc_macro2::{Literal, TokenStream, TokenTree};
use quote::quote;
use synstructure::decl_derive;

decl_derive!([FromRequest, attributes(nails)] => from_request_derive);

fn from_request_derive(s: synstructure::Structure) -> proc_macro2::TokenStream {
    // TODO: support path with variable (e.g. "/api/posts/{id}")
    let mut path: Option<String> = None;
    for meta in s
        .ast()
        .attrs
        .iter()
        .filter_map(|attr| attr.parse_meta().ok())
    {
        if meta.name() == "nails" {
            if let syn::Meta::List(ref list) = meta {
                if let Some(ref pair) = list.nested.first() {
                    if let &&syn::NestedMeta::Meta(syn::Meta::NameValue(ref nv)) = pair.value() {
                        if nv.ident == "path" {
                            if path.is_some() {
                                panic!("Cannot have two `path` attributes");
                            }
                            if let syn::Lit::Str(ref value) = nv.lit {
                                path = Some(value.value());
                            } else {
                                panic!("path must be a string");
                            }
                        }
                    }
                }
            }
        }
    }
    let path = path.expect("#[nails(path = \"\")] is needed");
    let path_lit = TokenTree::Literal(Literal::string(&path));

    assert_eq!(s.variants().len(), 1);
    let variant = &s.variants()[0];
    let construct = variant.construct(|field, idx| field_parser(field, idx));

    s.gen_impl(quote! {
        gen impl nails::FromRequest for @Self {
            fn path_prefix_hint() -> &'static str {
                #path_lit
            }
            fn match_path(method: &Method, path: &str) -> bool {
                // TODO: configurable method kind
                (*method == Method::GET || *method == Method::POST) && path == #path_lit
            }

            fn from_request(req: Request<Body>) -> Self {
                let query_hash = nails::request::parse_query(req.uri().query().unwrap_or(""));
                #construct
            }
        }
    })
}

fn field_parser(field: &syn::Field, _idx: usize) -> TokenStream {
    let mut query: Option<String> = None;
    for meta in field
        .attrs
        .iter()
        .filter_map(|attr| attr.parse_meta().ok())
    {
        if meta.name() == "nails" {
            if let syn::Meta::List(ref list) = meta {
                if let Some(ref pair) = list.nested.first() {
                    if let &&syn::NestedMeta::Meta(syn::Meta::NameValue(ref nv)) = pair.value() {
                        if nv.ident == "query" {
                            if query.is_some() {
                                panic!("Cannot have two `query` attributes");
                            }
                            if let syn::Lit::Str(ref value) = nv.lit {
                                query = Some(value.value());
                            } else {
                                panic!("query value must be a string");
                            }
                        }
                    } else if let &&syn::NestedMeta::Meta(syn::Meta::Word(ref name)) = pair.value()
                    {
                        if name == "query" {
                            if query.is_some() {
                                panic!("Cannot have two `query` attributes");
                            }
                            if let Some(ref ident) = field.ident {
                                query = Some(ident.to_string());
                            } else {
                                panic!("Specify name for this field");
                            }
                        }
                    }
                }
            }
        }
    }
    if let Some(ref query) = query {
        let query_name = TokenTree::Literal(Literal::string(query));
        quote! {
            nails::request::FromQuery::from_query(
                if let Some(values) = query_hash.get(#query_name) {
                    values.as_slice()
                } else {
                    &[]
                }
            ).unwrap()  // TODO: error handling
        }
    } else {
        panic!("FromRequest field must have #[nails(query)]");
    }
}
