extern crate proc_macro;

use proc_macro2::{Literal, TokenTree};
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
    s.gen_impl(quote! {
        gen impl nails::FromRequest for @Self {
            fn path_prefix_hint() -> &'static str {
                #path_lit
            }
            fn match_path(method: &Method, path: &str) -> bool {
                // TODO: configurable method kind
                (*method == Method::GET || *method == Method::POST) && path == #path_lit
            }

            fn from_request(_req: Request<Body>) -> Self {
                // TODO: field-wise construction
                Self {}
            }
        }
    })
}
