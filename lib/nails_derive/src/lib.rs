#![recursion_limit = "128"]
#![cfg_attr(feature = "proc_macro_diagnostics", feature(proc_macro_diagnostics))]

extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use synstructure::decl_derive;

use crate::attrs::{FieldAttrs, StructAttrs};
use crate::path::PathPattern;
use crate::utils::VariantInfoExt;

mod attrs;
mod path;
mod utils;

#[cfg(test)]
#[macro_use]
mod test_utils;

decl_derive!([FromRequest, attributes(nails)] => from_request_derive);

fn from_request_derive(s: synstructure::Structure) -> syn::Result<proc_macro2::TokenStream> {
    let attrs = StructAttrs::parse(&s.ast().attrs)?;
    let path = attrs
        .path
        .clone()
        .ok_or_else(|| syn::Error::new(s.ast().span(), "#[nails(path)] is needed"))?;
    let path = path
        .path
        .value()
        .parse::<PathPattern>()
        .map_err(|e| syn::Error::new(path.path.span(), e))?;
    let path_prefix = path.path_prefix();
    let path_condition = path.gen_path_condition(quote! { path });

    let variant = if let syn::Data::Struct(_) = &s.ast().data {
        assert_eq!(s.variants().len(), 1);
        &s.variants()[0]
    } else {
        return Err(syn::Error::new(
            s.ast().span(),
            "FromRequest cannot be derived for enums or unions",
        ));
    };
    let construct = variant.try_construct(|field, idx| field_parser(field, idx))?;

    Ok(s.gen_impl(quote! {
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
    }))
}

fn field_parser(field: &syn::Field, _idx: usize) -> syn::Result<TokenStream> {
    let attrs = FieldAttrs::parse(&field.attrs)?;
    if let Some(ref query) = attrs.query {
        let query_name = if let Some(query_name) = &query.name {
            query_name.value()
        } else if let Some(ident) = &field.ident {
            ident.to_string()
        } else {
            return Err(syn::Error::new(
                query.span,
                "Specify name with #[nails(query = \"\")]",
            ));
        };
        Ok(quote! {
            nails::request::FromQuery::from_query(
                if let Some(values) = query_hash.get(#query_name) {
                    values.as_slice()
                } else {
                    &[]
                }
            ).unwrap()  // TODO: error handling
        })
    } else {
        return Err(syn::Error::new(
            field.span(),
            "FromRequest field must have #[nails(query)]",
        ));
    }
}

#[cfg(test)]
#[cfg_attr(tarpaulin, skip)]
mod tests {
    use super::*;

    #[test]
    fn test_derive1() {
        let ast = syn::parse2::<syn::DeriveInput>(quote! {
            #[nails(path = "/api/posts/{id}")]
            struct GetPostRequest {
                #[nails(query)]
                param1: String,
                #[nails(query = "param2rename")]
                param2: String,
            }
        })
        .unwrap();
        let s = synstructure::Structure::new(&ast);
        assert_ts_eq!(
            from_request_derive(s).unwrap(),
            quote! {
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
            },
        );
    }

    #[test]
    #[should_panic(expected = "FromRequest cannot be derived for enums or unions")]
    fn test_derive_enum() {
        let ast = syn::parse2::<syn::DeriveInput>(quote! {
            #[nails(path = "/api/posts/{id}")]
            enum GetPostRequest {
                Foo {}
            }
        })
        .unwrap();
        let s = synstructure::Structure::new(&ast);
        from_request_derive(s).unwrap();
    }

    #[test]
    #[should_panic(expected = "multiple #[nails(path)] definitions")]
    fn test_derive_double_paths() {
        let ast = syn::parse2::<syn::DeriveInput>(quote! {
            #[nails(path = "/api/posts/{id}")]
            #[nails(path = "/api/posts/{idd}")]
            struct GetPostRequest {}
        })
        .unwrap();
        let s = synstructure::Structure::new(&ast);
        from_request_derive(s).unwrap();
    }

    #[test]
    #[should_panic(expected = "multiple #[nails(path)] definitions")]
    fn test_derive_double_paths2() {
        let ast = syn::parse2::<syn::DeriveInput>(quote! {
            #[nails(path = "/api/posts/{id}", path = "/api/posts/{idd}")]
            struct GetPostRequest {}
        })
        .unwrap();
        let s = synstructure::Structure::new(&ast);
        from_request_derive(s).unwrap();
    }

    #[test]
    #[should_panic(expected = "string value expected in #[nails(path)]")]
    fn test_derive_integer_path() {
        let ast = syn::parse2::<syn::DeriveInput>(quote! {
            #[nails(path = 1)]
            struct GetPostRequest {}
        })
        .unwrap();
        let s = synstructure::Structure::new(&ast);
        from_request_derive(s).unwrap();
    }

    #[test]
    #[should_panic(expected = "#[nails(path)] is needed")]
    fn test_derive_missing_path() {
        let ast = syn::parse2::<syn::DeriveInput>(quote! {
            struct GetPostRequest {}
        })
        .unwrap();
        let s = synstructure::Structure::new(&ast);
        from_request_derive(s).unwrap();
    }

    #[test]
    #[should_panic(expected = "unknown option: `foo`")]
    fn test_derive_unknown_struct_attr() {
        let ast = syn::parse2::<syn::DeriveInput>(quote! {
            #[nails(path = "/api/posts/{id}", foo)]
            struct GetPostRequest {}
        })
        .unwrap();
        let s = synstructure::Structure::new(&ast);
        from_request_derive(s).unwrap();
    }

    #[test]
    #[should_panic(expected = "multiple #[nails(query)] definitions")]
    fn test_derive_double_queries() {
        let ast = syn::parse2::<syn::DeriveInput>(quote! {
            #[nails(path = "/api/posts/{id}")]
            struct GetPostRequest {
                #[nails(query = "query1rename")]
                #[nails(query = "query1renamerename")]
                query1: String,
            }
        })
        .unwrap();
        let s = synstructure::Structure::new(&ast);
        from_request_derive(s).unwrap();
    }

    #[test]
    #[should_panic(expected = "multiple #[nails(query)] definitions")]
    fn test_derive_double_queries2() {
        let ast = syn::parse2::<syn::DeriveInput>(quote! {
            #[nails(path = "/api/posts/{id}")]
            struct GetPostRequest {
                #[nails(query = "query1rename", query = "query1renamerename")]
                query1: String,
            }
        })
        .unwrap();
        let s = synstructure::Structure::new(&ast);
        from_request_derive(s).unwrap();
    }

    #[test]
    #[should_panic(expected = "string value or no value expected in #[nails(query)]")]
    fn test_derive_integer_query_name() {
        let ast = syn::parse2::<syn::DeriveInput>(quote! {
            #[nails(path = "/api/posts/{id}")]
            struct GetPostRequest {
                #[nails(query = 1)]
                query1: String,
            }
        })
        .unwrap();
        let s = synstructure::Structure::new(&ast);
        from_request_derive(s).unwrap();
    }

    #[test]
    #[should_panic(expected = "Specify name with #[nails(query = \\\"\\\")]")]
    fn test_derive_missing_query_name_for_position_field() {
        let ast = syn::parse2::<syn::DeriveInput>(quote! {
            #[nails(path = "/api/posts/{id}")]
            struct GetPostRequest(
                #[nails(query)]
                String,
            );
        })
        .unwrap();
        let s = synstructure::Structure::new(&ast);
        from_request_derive(s).unwrap();
    }

    #[test]
    #[should_panic(expected = "FromRequest field must have #[nails(query)]")]
    fn test_derive_missing_query_attr() {
        let ast = syn::parse2::<syn::DeriveInput>(quote! {
            #[nails(path = "/api/posts/{id}")]
            struct GetPostRequest {
                query1: String,
            }
        })
        .unwrap();
        let s = synstructure::Structure::new(&ast);
        from_request_derive(s).unwrap();
    }

    #[test]
    #[should_panic(expected = "unknown option: `foo`")]
    fn test_derive_unknown_field_attr() {
        let ast = syn::parse2::<syn::DeriveInput>(quote! {
            #[nails(path = "/api/posts/{id}")]
            struct GetPostRequest {
                #[nails(query, foo)]
                query1: String,
            }
        })
        .unwrap();
        let s = synstructure::Structure::new(&ast);
        from_request_derive(s).unwrap();
    }
}
