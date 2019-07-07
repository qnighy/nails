#![recursion_limit = "128"]
#![cfg_attr(feature = "proc_macro_diagnostics", feature(proc_macro_diagnostics))]

extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::DeriveInput;

use crate::attrs::{FieldAttrs, StructAttrs};
use crate::path::PathPattern;
use crate::utils::FieldsExt;

mod attrs;
mod path;
mod utils;

#[cfg(test)]
#[macro_use]
mod test_utils;

#[proc_macro_derive(FromRequest, attributes(nails))]
pub fn derive_from_request(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_from_request2(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

fn derive_from_request2(input: TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse2::<DeriveInput>(input)?;
    let attrs = StructAttrs::parse(&input.attrs)?;
    let path = attrs
        .path
        .clone()
        .ok_or_else(|| syn::Error::new(input.span(), "#[nails(path)] is needed"))?;
    let path = path
        .path
        .value()
        .parse::<PathPattern>()
        .map_err(|e| syn::Error::new(path.path.span(), e))?;
    let path_prefix = path.path_prefix();
    let path_condition = path.gen_path_condition(quote! { path });

    let data = if let syn::Data::Struct(data) = &input.data {
        data
    } else {
        return Err(syn::Error::new(
            input.span(),
            "FromRequest cannot be derived for enums or unions",
        ));
    };
    let construct = data
        .fields
        .try_construct(&input.ident, |field, idx| field_parser(field, idx))?;

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = &input.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics nails::FromRequest for #name #ty_generics #where_clause {
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
        assert_ts_eq!(
            derive_from_request2(quote! {
                #[nails(path = "/api/posts/{id}")]
                struct GetPostRequest {
                    #[nails(query)]
                    param1: String,
                    #[nails(query = "param2rename")]
                    param2: String,
                }
            })
            .unwrap(),
            quote! {
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
            },
        );
    }

    #[test]
    #[should_panic(expected = "FromRequest cannot be derived for enums or unions")]
    fn test_derive_enum() {
        derive_from_request2(quote! {
            #[nails(path = "/api/posts/{id}")]
            enum GetPostRequest {
                Foo {}
            }
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "multiple #[nails(path)] definitions")]
    fn test_derive_double_paths() {
        derive_from_request2(quote! {
            #[nails(path = "/api/posts/{id}")]
            #[nails(path = "/api/posts/{idd}")]
            struct GetPostRequest {}
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "multiple #[nails(path)] definitions")]
    fn test_derive_double_paths2() {
        derive_from_request2(quote! {
            #[nails(path = "/api/posts/{id}", path = "/api/posts/{idd}")]
            struct GetPostRequest {}
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "string value expected in #[nails(path)]")]
    fn test_derive_integer_path() {
        derive_from_request2(quote! {
            #[nails(path = 1)]
            struct GetPostRequest {}
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "#[nails(path)] is needed")]
    fn test_derive_missing_path() {
        derive_from_request2(quote! {
            struct GetPostRequest {}
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "unknown option: `foo`")]
    fn test_derive_unknown_struct_attr() {
        derive_from_request2(quote! {
            #[nails(path = "/api/posts/{id}", foo)]
            struct GetPostRequest {}
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "multiple #[nails(query)] definitions")]
    fn test_derive_double_queries() {
        derive_from_request2(quote! {
            #[nails(path = "/api/posts/{id}")]
            struct GetPostRequest {
                #[nails(query = "query1rename")]
                #[nails(query = "query1renamerename")]
                query1: String,
            }
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "multiple #[nails(query)] definitions")]
    fn test_derive_double_queries2() {
        derive_from_request2(quote! {
            #[nails(path = "/api/posts/{id}")]
            struct GetPostRequest {
                #[nails(query = "query1rename", query = "query1renamerename")]
                query1: String,
            }
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "string value or no value expected in #[nails(query)]")]
    fn test_derive_integer_query_name() {
        derive_from_request2(quote! {
            #[nails(path = "/api/posts/{id}")]
            struct GetPostRequest {
                #[nails(query = 1)]
                query1: String,
            }
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "Specify name with #[nails(query = \\\"\\\")]")]
    fn test_derive_missing_query_name_for_position_field() {
        derive_from_request2(quote! {
            #[nails(path = "/api/posts/{id}")]
            struct GetPostRequest(
                #[nails(query)]
                String,
            );
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "FromRequest field must have #[nails(query)]")]
    fn test_derive_missing_query_attr() {
        derive_from_request2(quote! {
            #[nails(path = "/api/posts/{id}")]
            struct GetPostRequest {
                query1: String,
            }
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "unknown option: `foo`")]
    fn test_derive_unknown_field_attr() {
        derive_from_request2(quote! {
            #[nails(path = "/api/posts/{id}")]
            struct GetPostRequest {
                #[nails(query, foo)]
                query1: String,
            }
        })
        .unwrap();
    }
}
