#![recursion_limit = "128"]
#![cfg_attr(feature = "proc_macro_diagnostics", feature(proc_macro_diagnostics))]

extern crate proc_macro;

use std::collections::{HashMap, HashSet};

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

    let data = if let syn::Data::Struct(data) = &input.data {
        data
    } else {
        return Err(syn::Error::new(
            input.span(),
            "FromRequest cannot be derived for enums or unions",
        ));
    };
    let attrs = StructAttrs::parse(&input.attrs)?;
    let field_attrs = data
        .fields
        .iter()
        .map(|field| FieldAttrs::parse(&field.attrs))
        .collect::<Result<Vec<_>, _>>()?;

    let path = attrs
        .path
        .clone()
        .ok_or_else(|| syn::Error::new(input.span(), "#[nails(path)] is needed"))?;
    let path_span = path.path.span();
    let path = path
        .path
        .value()
        .parse::<PathPattern>()
        .map_err(|e| syn::Error::new(path_span, e))?;

    let field_kinds = data
        .fields
        .iter()
        .zip(&field_attrs)
        .map(|(field, attrs)| FieldKind::parse_from(field, attrs, path.bindings()))
        .collect::<Result<Vec<_>, _>>()?;

    let path_fields = {
        let mut path_fields = HashMap::new();
        for (idx, field) in data.fields.iter().enumerate() {
            if let FieldKind::Path { var } = &field_kinds[idx] {
                if let Some(_dup_field) = path_fields.get(var) {
                    let span = if let Some(path) = &field_attrs[idx].path {
                        path.span
                    } else {
                        field.span()
                    };
                    return Err(syn::Error::new(span, "Duplicate path names"));
                }
                path_fields.insert(var.clone(), field);
            }
        }
        path_fields
    };
    for binding in path.bindings() {
        if !path_fields.contains_key(binding) {
            return Err(syn::Error::new(
                path_span,
                format_args!("Missing field for binding name from {{{}}}", binding),
            ));
        }
    }

    let path_prefix = path.path_prefix();
    let path_condition = path.gen_path_condition(quote! { path });

    let construct = data.fields.try_construct(&input.ident, |field, idx| {
        field_kinds[idx].gen_parser(field)
    })?;

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

#[derive(Debug)]
enum FieldKind {
    Path { var: String },
    Query { name: String },
}

impl FieldKind {
    fn parse_from(
        field: &syn::Field,
        attrs: &FieldAttrs,
        path_bindings: &HashSet<String>,
    ) -> syn::Result<FieldKind> {
        if let Some(query) = &attrs.query {
            if attrs.path.is_some() {
                return Err(syn::Error::new(
                    query.span,
                    "Cannot have both #[nails(query)] and #[nails(path)]",
                ));
            }
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
            return Ok(FieldKind::Query { name: query_name });
        }

        if let Some(path) = &attrs.path {
            let path_name = if let Some(path_name) = &path.name {
                path_name.value()
            } else if let Some(ident) = &field.ident {
                ident.to_string()
            } else {
                return Err(syn::Error::new(
                    path.span,
                    "Specify name with #[nails(path = \"\")]",
                ));
            };
            if !path_bindings.contains(&path_name) {
                return Err(syn::Error::new(
                    path.span,
                    "This name doesn't exist in the endpoint path",
                ));
            }
            return Ok(FieldKind::Path { var: path_name });
        }

        // ident-based fallback
        let ident = field.ident.as_ref().ok_or_else(|| {
            syn::Error::new(
                field.span(),
                "Specify name with #[nails(query = \"\")] or alike",
            )
        })?;
        let ident_name = ident.to_string();
        if path_bindings.contains(&ident_name) {
            // fallback to path
            Ok(FieldKind::Path { var: ident_name })
        } else {
            // fallback to query
            Ok(FieldKind::Query { name: ident_name })
        }
    }

    fn gen_parser(&self, _field: &syn::Field) -> syn::Result<TokenStream> {
        Ok(match self {
            FieldKind::Path { var: _var } => quote! {
                unimplemented!() // TODO: handle paths
            },
            FieldKind::Query { name } => quote! {
                nails::request::FromQuery::from_query(
                    if let Some(values) = query_hash.get(#name) {
                        values.as_slice()
                    } else {
                        &[]
                    }
                ).unwrap()  // TODO: error handling
            },
        })
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
                    id: String,
                    #[nails(query)]
                    param1: String,
                    #[nails(query = "param2rename")]
                    param2: String,
                    param3: String,
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
                            id: unimplemented!(),
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
                            param3: nails::request::FromQuery::from_query(
                                if let Some(values) = query_hash.get("param3") {
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
    fn test_derive_tuple() {
        assert_ts_eq!(
            derive_from_request2(quote! {
                #[nails(path = "/api/posts/{id}")]
                struct GetPostRequest(
                    #[nails(path = "id")]
                    String,
                    #[nails(query = "param1")]
                    String,
                );
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
                        Ok(GetPostRequest(
                            unimplemented!(),
                            nails::request::FromQuery::from_query(
                                if let Some(values) = query_hash.get("param1") {
                                    values.as_slice()
                                } else {
                                    &[]
                                }
                            ).unwrap(),
                        ))
                    }
                }
            },
        );
    }

    #[test]
    fn test_derive_unit() {
        assert_ts_eq!(
            derive_from_request2(quote! {
                #[nails(path = "/ping")]
                struct PingRequest;
            })
            .unwrap(),
            quote! {
                impl nails::FromRequest for PingRequest {
                    fn path_prefix_hint() -> &'static str { "/ping" }
                    fn match_path(method: &Method, path: &str) -> bool {
                        (*method == Method::GET || *method == Method::HEAD) && (
                            path.starts_with("/") && {
                                let mut path_iter = path[1..].split("/");
                                path_iter.next().map(|comp| comp == "ping").unwrap_or(false)
                                    && path_iter.next().is_none()
                            }
                        )
                    }
                    fn from_request(req: Request<Body>) -> Result<Self, nails::response::ErrorResponse> {
                        let query_hash = nails::request::parse_query(req.uri().query().unwrap_or(""));
                        Ok(PingRequest)
                    }
                }
            },
        );
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
    #[should_panic(expected = "multiple #[nails(path)] definitions")]
    fn test_derive_double_path_kinds() {
        derive_from_request2(quote! {
            #[nails(path = "/api/posts/{id1}/{id2}")]
            struct GetPostRequest {
                #[nails(path = "id1", path = "id2")]
                id: String,
            }
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "Duplicate path names")]
    fn test_derive_duplicate_path_names() {
        derive_from_request2(quote! {
            #[nails(path = "/api/posts/{id}")]
            struct GetPostRequest {
                #[nails(path)]
                id: String,
                #[nails(path = "id")]
                id2: String,
            }
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "Duplicate path names")]
    fn test_derive_duplicate_path_names2() {
        derive_from_request2(quote! {
            #[nails(path = "/api/posts/{id}")]
            struct GetPostRequest {
                #[nails(path = "id")]
                id2: String,
                id: String,
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
    #[should_panic(expected = "string value or no value expected in #[nails(path)]")]
    fn test_derive_integer_path_name() {
        derive_from_request2(quote! {
            #[nails(path = "/api/posts/{id}")]
            struct GetPostRequest {
                #[nails(path = 1)]
                id: String,
            }
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "This name doesn\\'t exist in the endpoint path")]
    fn test_derive_non_captured_path_name1() {
        derive_from_request2(quote! {
            #[nails(path = "/api/posts/{id}")]
            struct GetPostRequest {
                #[nails(path = "idd")]
                id: String,
            }
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "This name doesn\\'t exist in the endpoint path")]
    fn test_derive_non_captured_path_name2() {
        derive_from_request2(quote! {
            #[nails(path = "/api/posts/{id}")]
            struct GetPostRequest {
                #[nails(path)]
                idd: String,
            }
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "Missing field for binding name from {id}")]
    fn test_derive_missing_path_names() {
        derive_from_request2(quote! {
            #[nails(path = "/api/posts/{id}")]
            struct GetPostRequest;
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "Specify name with #[nails(query = \\\"\\\")] or alike")]
    fn test_derive_missing_field_kind_for_position_field() {
        derive_from_request2(quote! {
            #[nails(path = "/api/posts/{id}")]
            struct GetPostRequest(
                String,
            );
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
    #[should_panic(expected = "Specify name with #[nails(path = \\\"\\\")]")]
    fn test_derive_missing_path_name_for_position_field() {
        derive_from_request2(quote! {
            #[nails(path = "/api/posts/{id}")]
            struct GetPostRequest(
                #[nails(path)]
                String,
            );
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
