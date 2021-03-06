use std::collections::{HashMap, HashSet};
use std::fmt;
use std::str::FromStr;

use proc_macro2::{Span, TokenStream};
use quote::quote;

// TODO: support recursive glob like `/admin/sidekiq/{path*}`
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PathPattern {
    components: Vec<ComponentMatcher>,
    bindings: HashSet<String>,
}

impl PathPattern {
    pub(crate) fn path_prefix(&self) -> String {
        let mut prefix = String::from("");
        for comp in &self.components {
            match comp {
                ComponentMatcher::String(s) => {
                    prefix.push_str("/");
                    prefix.push_str(s);
                }
                ComponentMatcher::Var(_) => {
                    prefix.push_str("/");
                    return prefix;
                }
            }
        }
        prefix
    }

    pub(crate) fn gen_path_condition(
        &self,
        path: TokenStream,
        fields: &HashMap<String, &syn::Field>,
    ) -> TokenStream {
        let conditions = self
            .components
            .iter()
            .map(|comp| match comp {
                ComponentMatcher::String(s) => {
                    quote! {
                        path_iter.next().map(|comp| comp == #s).unwrap_or(false) &&
                    }
                }
                ComponentMatcher::Var(var) => {
                    let field_ty = &fields[var].ty;
                    quote! {
                        path_iter.next().map(|comp| {
                            <#field_ty as nails::__rt::FromPath>::matches(comp)
                        }).unwrap_or(false) &&
                    }
                }
            })
            .collect::<TokenStream>();
        quote! {(
            #path.starts_with("/") && {
                let mut path_iter = #path[1..].split("/");
                #conditions
                path_iter.next().is_none()
            }
        )}
    }

    pub(crate) fn gen_path_extractor(
        &self,
        path: TokenStream,
        fields: &HashMap<String, &syn::Field>,
    ) -> (TokenStream, HashMap<String, syn::Ident>) {
        let mut vars = HashMap::new();
        let extractors = self
            .components
            .iter()
            .map(|comp| match comp {
                ComponentMatcher::String(_) => {
                    quote! {
                        path_iter.next();
                    }
                }
                ComponentMatcher::Var(var) => {
                    let var_ident = format!("pathcomp_{}", var);
                    let var_ident = syn::Ident::new(&var_ident, Span::call_site());
                    vars.insert(var.clone(), var_ident.clone());
                    let field_ty = &fields[var].ty;
                    quote! {
                        let #var_ident = <#field_ty as nails::__rt::FromPath>::from_path(
                            path_iter.next().expect("internal error: invalid path given")
                        ).expect("internal error: invalid path given");
                    }
                }
            })
            .collect::<TokenStream>();
        let extractor = quote! {
            let mut path_iter = #path[1..].split("/");
            #extractors
        };
        (extractor, vars)
    }

    pub(crate) fn bindings(&self) -> &HashSet<String> {
        &self.bindings
    }
}

impl FromStr for PathPattern {
    type Err = ParseError;

    fn from_str(path: &str) -> Result<Self, Self::Err> {
        if !path.starts_with("/") {
            return Err(ParseError::new(path, "must start with slash"));
        }
        let components = path[1..]
            .split("/")
            .map(|c| -> Result<_, Self::Err> {
                // TODO: support `{{`, `}}`
                if c.contains("{") || c.contains("}") {
                    if !c.starts_with("{") || !c.ends_with("}") {
                        return Err(ParseError::new(
                            path,
                            "variable must span the whole path component",
                        ));
                    }
                    let c = &c[1..c.len() - 1];
                    if c.contains("{") || c.contains("}") {
                        return Err(ParseError::new(
                            path,
                            "variable must span the whole path component",
                        ));
                    }
                    if c == "" {
                        return Err(ParseError::new(path, "variable must contain variable name"));
                    }
                    if !is_ident(c) {
                        return Err(ParseError::new(
                            path,
                            "variable must be /[a-zA-Z_][a-zA-Z0-9_]*/",
                        ));
                    }
                    Ok(ComponentMatcher::Var(c.to_owned()))
                } else {
                    // TODO: more sanity check
                    Ok(ComponentMatcher::String(c.to_owned()))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut bindings = HashSet::new();
        for comp in &components {
            match comp {
                ComponentMatcher::String(_) => {}
                ComponentMatcher::Var(name) => {
                    let success = bindings.insert(name.clone());
                    if !success {
                        return Err(ParseError::new(
                            path,
                            &format!("duplicate name: `{}`", name),
                        ));
                    }
                }
            }
        }

        Ok(Self {
            components,
            bindings,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ComponentMatcher {
    String(String),
    Var(String),
}

#[derive(Debug, Clone)]
pub(crate) struct ParseError {
    path: String,
    message: String,
}

impl ParseError {
    fn new(path: &str, message: &str) -> Self {
        Self {
            path: path.to_owned(),
            message: message.to_owned(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "error while parsing path matcher `{}`: {}",
            self.path, self.message
        )
    }
}

impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        "error while parsing path matcher"
    }
}

fn is_ident(s: &str) -> bool {
    let s = s.as_bytes();
    s.len() > 0
        && s != b"_"
        && (s[0].is_ascii_alphabetic() || s[0] == b'_')
        && s.iter().all(|&c| c.is_ascii_alphanumeric() || c == b'_')
}

#[cfg(test)]
#[cfg_attr(tarpaulin, skip)]
mod tests {
    use super::*;

    use crate::assert_ts_eq;
    use syn::parse::Parser;

    macro_rules! hash {
        [$($e:expr),*] => {
            vec![$($e,)*].into_iter().collect::<HashMap<_, _>>()
        };
        [$($e:expr)*,] => {
            vec![$($e,)*].into_iter().collect::<HashMap<_, _>>()
        };
    }
    macro_rules! hash_set {
        [$($e:expr),*] => {
            vec![$($e,)*].into_iter().collect::<HashSet<_>>()
        };
        [$($e:expr)*,] => {
            vec![$($e,)*].into_iter().collect::<HashSet<_>>()
        };
    }

    #[test]
    fn test_is_ident() {
        assert!(is_ident("foo_bar2"));
        assert!(is_ident("_foo_bar2"));
        assert!(!is_ident("_"));
        assert!(!is_ident("1st"));
        assert!(!is_ident("foo-bar"));
    }

    #[test]
    fn test_path_prefix() {
        let parse = <PathPattern as FromStr>::from_str;
        let parse = |s| parse(s).unwrap();
        assert_eq!(parse("/").path_prefix(), "/");
        assert_eq!(parse("/api/posts/{id}").path_prefix(), "/api/posts/");
        assert_eq!(
            parse("/api/posts/{post_id}/comments/{id}").path_prefix(),
            "/api/posts/",
        );
    }

    #[test]
    fn test_gen_path_condition() {
        let parse = <PathPattern as FromStr>::from_str;
        let parse = |s| parse(s).unwrap();
        assert_ts_eq!(
            parse("/").gen_path_condition(quote! { path }, &hash![]),
            quote! {
                (path.starts_with("/") && {
                    let mut path_iter = path[1..].split("/");
                    path_iter.next().map(|comp| comp == "").unwrap_or(false)
                        && path_iter.next().is_none()
                })
            },
        );

        let field = syn::Field::parse_named
            .parse2(quote! { id: String })
            .unwrap();
        assert_ts_eq!(
            parse("/api/posts/{id}")
                .gen_path_condition(quote! { path }, &hash![(S("id"), &field),]),
            quote! {
                (path.starts_with("/") && {
                    let mut path_iter = path[1..].split("/");
                    path_iter.next().map(|comp| comp == "api").unwrap_or(false)
                        && path_iter.next().map(|comp| comp == "posts").unwrap_or(false)
                        && path_iter.next().map(|comp| {
                            <String as nails::__rt::FromPath>::matches(comp)
                        }).unwrap_or(false)
                        && path_iter.next().is_none()
                })
            },
        );
    }

    #[test]
    fn test_gen_path_extractor() {
        let parse = <PathPattern as FromStr>::from_str;
        let parse = |s| parse(s).unwrap();

        let (extractor, vars) = parse("/").gen_path_extractor(quote! { path }, &hash![]);
        assert_ts_eq!(
            extractor,
            quote! {
                let mut path_iter = path[1..].split("/");
                path_iter.next();
            },
        );
        assert_eq!(vars.len(), 0);

        let field = syn::Field::parse_named
            .parse2(quote! { id: String })
            .unwrap();
        let (extractor, vars) = parse("/api/posts/{id}")
            .gen_path_extractor(quote! { path }, &hash![(S("id"), &field),]);
        assert_ts_eq!(
            extractor,
            quote! {
                let mut path_iter = path[1..].split("/");
                path_iter.next();
                path_iter.next();
                let pathcomp_id = <String as nails::__rt::FromPath>::from_path(
                    path_iter.next().expect("internal error: invalid path given")
                ).expect("internal error: invalid path given");
            },
        );
        assert_eq!(vars.len(), 1);
        assert_eq!(vars["id"], "pathcomp_id");
    }

    #[test]
    fn test_parse() {
        let parse = <PathPattern as FromStr>::from_str;
        let parse = |s| parse(s).unwrap();
        assert_eq!(
            parse("/"),
            PathPattern {
                components: vec![ComponentMatcher::String(S("")),],
                bindings: hash_set![],
            },
        );
        assert_eq!(
            parse("/ping"),
            PathPattern {
                components: vec![ComponentMatcher::String(S("ping")),],
                bindings: hash_set![],
            },
        );
        assert_eq!(
            parse("/api/"),
            PathPattern {
                components: vec![
                    ComponentMatcher::String(S("api")),
                    ComponentMatcher::String(S("")),
                ],
                bindings: hash_set![],
            },
        );
        assert_eq!(
            parse("/api/posts/{id}"),
            PathPattern {
                components: vec![
                    ComponentMatcher::String(S("api")),
                    ComponentMatcher::String(S("posts")),
                    ComponentMatcher::Var(S("id")),
                ],
                bindings: hash_set![S("id")],
            },
        );
        assert_eq!(
            parse("/api/posts/{post_id}/comments/{id}"),
            PathPattern {
                components: vec![
                    ComponentMatcher::String(S("api")),
                    ComponentMatcher::String(S("posts")),
                    ComponentMatcher::Var(S("post_id")),
                    ComponentMatcher::String(S("comments")),
                    ComponentMatcher::Var(S("id")),
                ],
                bindings: hash_set![S("post_id"), S("id")],
            },
        );
    }

    #[test]
    fn test_parse_error() {
        let parse = <PathPattern as FromStr>::from_str;
        let parse_err = |s| parse(s).unwrap_err().message;
        assert_eq!(parse_err(""), "must start with slash");
        assert_eq!(parse_err("api/posts/{id}"), "must start with slash",);
        assert_eq!(
            parse_err("/api/posts/post_{id}"),
            "variable must span the whole path component",
        );
        assert_eq!(
            parse_err("/api/posts/{foo}_{bar}"),
            "variable must span the whole path component",
        );
        assert_eq!(
            parse_err("/api/posts/}/"),
            "variable must span the whole path component",
        );
        assert_eq!(
            parse_err("/api/posts/{barrr/"),
            "variable must span the whole path component",
        );
        assert_eq!(
            parse_err("/api/posts/}foo{/"),
            "variable must span the whole path component",
        );
        assert_eq!(
            parse_err("/api/posts/{}/"),
            "variable must contain variable name",
        );
        assert_eq!(
            parse_err("/api/posts/{1}/"),
            "variable must be /[a-zA-Z_][a-zA-Z0-9_]*/",
        );
        assert_eq!(
            parse_err("/api/posts/{id}/comments/{id}"),
            "duplicate name: `id`",
        );
    }

    #[test]
    fn test_parse_error_message() {
        let parse = <PathPattern as FromStr>::from_str;
        let parse_err = |s| parse(s).unwrap_err().to_string();
        assert_eq!(
            parse_err(""),
            "error while parsing path matcher ``: must start with slash",
        );
        assert_eq!(
            parse_err("api/posts/{id}"),
            "error while parsing path matcher `api/posts/{id}`: must start with slash",
        );

        {
            use std::error::Error as _;
            assert_eq!(
                parse("").unwrap_err().description(),
                "error while parsing path matcher",
            );
        }
    }

    #[allow(non_snake_case)]
    fn S(s: &'static str) -> String {
        s.to_owned()
    }
}
