#![recursion_limit = "128"]
#![cfg_attr(feature = "proc_macro_diagnostics", feature(proc_macro_diagnostics))]

extern crate proc_macro;

use proc_macro2::{Literal, TokenStream, TokenTree};
use quote::quote;
use synstructure::decl_derive;

use crate::attrs::StructAttrs;
use crate::path::PathPattern;

mod attrs;
mod path;

decl_derive!([FromRequest, attributes(nails)] => from_request_derive);

fn from_request_derive(s: synstructure::Structure) -> proc_macro2::TokenStream {
    let attrs = StructAttrs::parse(&s.ast().attrs).unwrap();
    let path = attrs.path.clone().expect("#[nails(path = \"\")] is needed");
    let path = path.path.value();
    let path = path.parse::<PathPattern>().unwrap();
    let path_prefix = path.path_prefix();
    let path_condition = path.gen_path_condition(quote! { path });

    assert_eq!(s.variants().len(), 1);
    let variant = &s.variants()[0];
    let construct = variant.construct(|field, idx| field_parser(field, idx));

    s.gen_impl(quote! {
        gen impl nails::FromRequest for @Self {
            fn path_prefix_hint() -> &'static str {
                #path_prefix
            }
            fn match_path(method: &Method, path: &str) -> bool {
                // TODO: configurable method kind
                (*method == Method::GET || *method == Method::HEAD) && #path_condition
            }

            fn from_request(req: Request<Body>) -> Result<Self, nails::response::ErrorResponse> {
                let query_hash = nails::request::parse_query(req.uri().query().unwrap_or(""));
                Ok(#construct)
            }
        }
    })
}

fn field_parser(field: &syn::Field, _idx: usize) -> TokenStream {
    let mut query: Option<String> = None;
    for meta in field.attrs.iter().filter_map(|attr| attr.parse_meta().ok()) {
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

#[cfg(test)]
#[cfg_attr(tarpaulin, skip)]
mod tests {
    use super::*;

    use synstructure::test_derive;

    #[test]
    fn test_derive1() {
        test_derive! {
            from_request_derive {
                #[nails(path = "/api/posts/{id}")]
                struct GetPostRequest {
                    #[nails(query)]
                    param1: String,
                    #[nails(query = "param2rename")]
                    param2: String,
                }
            }
            expands to {
                #[allow(non_upper_case_globals)]
                const _DERIVE_nails_FromRequest_FOR_GetPostRequest: () = {
                    impl nails::FromRequest for GetPostRequest {
                        fn path_prefix_hint() -> &'static str { "/api/posts/" }
                        fn match_path(method: &Method, path: &str) -> bool {
                            (*method == Method::GET || *method == Method::HEAD) && (
                                path.starts_with("/") && {
                                    let mut path_iter = path[1..].split("/");
                                    path_iter.next().map(|comp| comp == "api").unwrap_or(false)
                                        && path_iter.next().map(|comp| comp == "posts").unwrap_or(false)
                                        && path_iter.next().is_some()
                                        && path_iter.next().is_none()
                                }
                            )
                        }
                        fn from_request(req: Request<Body>) -> Result<Self, nails::response::ErrorResponse> {
                            let query_hash = nails::request::parse_query(req.uri().query().unwrap_or(""));
                            Ok(GetPostRequest {
                                param1: nails::request::FromQuery::from_query(
                                    if let Some(values) = query_hash.get("param1") {
                                        values.as_slice()
                                    } else {
                                        &[]
                                    }
                                ).unwrap(),
                                param2: nails::request::FromQuery::from_query(
                                    if let Some(values) = query_hash.get("param2rename") {
                                        values.as_slice()
                                    } else {
                                        &[]
                                    }
                                ).unwrap(),
                            })
                        }
                    }
                };
            }
            no_build
        }
    }

    #[test]
    #[should_panic(expected = "multiple #[nails(path)] definitions")]
    fn test_derive_double_paths() {
        test_derive! {
            from_request_derive {
                #[nails(path = "/api/posts/{id}")]
                #[nails(path = "/api/posts/{idd}")]
                struct GetPostRequest {}
            }
            expands to {}
            no_build
        }
    }

    #[test]
    #[should_panic(expected = "multiple #[nails(path)] definitions")]
    fn test_derive_double_paths2() {
        test_derive! {
            from_request_derive {
                #[nails(path = "/api/posts/{id}", path = "/api/posts/{idd}")]
                struct GetPostRequest {}
            }
            expands to {}
            no_build
        }
    }

    #[test]
    #[should_panic(expected = "string value expected in #[nails(path)]")]
    fn test_derive_integer_path() {
        test_derive! {
            from_request_derive {
                #[nails(path = 1)]
                struct GetPostRequest {}
            }
            expands to {}
            no_build
        }
    }

    #[test]
    #[should_panic(expected = "#[nails(path = \"\")] is needed")]
    fn test_derive_missing_path() {
        test_derive! {
            from_request_derive {
                struct GetPostRequest {}
            }
            expands to {}
            no_build
        }
    }

    #[test]
    #[should_panic(expected = "Cannot have two `query` attributes")]
    fn test_derive_double_queries() {
        test_derive! {
            from_request_derive {
                #[nails(path = "/api/posts/{id}")]
                struct GetPostRequest {
                    #[nails(query = "query1rename")]
                    #[nails(query = "query1renamerename")]
                    query1: String,
                }
            }
            expands to {}
            no_build
        }
    }

    #[test]
    #[should_panic(expected = "query value must be a string")]
    fn test_derive_integer_query_name() {
        test_derive! {
            from_request_derive {
                #[nails(path = "/api/posts/{id}")]
                struct GetPostRequest {
                    #[nails(query = 1)]
                    query1: String,
                }
            }
            expands to {}
            no_build
        }
    }

    #[test]
    #[should_panic(expected = "Specify name for this field")]
    fn test_derive_missing_query_name_for_position_field() {
        test_derive! {
            from_request_derive {
                #[nails(path = "/api/posts/{id}")]
                struct GetPostRequest(
                    #[nails(query)]
                    String,
                );
            }
            expands to {}
            no_build
        }
    }

    #[test]
    #[should_panic(expected = "FromRequest field must have #[nails(query)]")]
    fn test_derive_missing_query_attr() {
        test_derive! {
            from_request_derive {
                #[nails(path = "/api/posts/{id}")]
                struct GetPostRequest {
                    query1: String,
                }
            }
            expands to {}
            no_build
        }
    }
}
