use std::fmt;
use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::quote;

// TODO: support recursive glob like `/admin/sidekiq/{path*}`
#[derive(Debug, Clone)]
pub(crate) struct PathPattern {
    components: Vec<ComponentMatcher>,
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

    pub(crate) fn gen_path_condition(&self, path: TokenStream) -> TokenStream {
        let conditions = self
            .components
            .iter()
            .map(|comp| match comp {
                ComponentMatcher::String(s) => {
                    quote! {
                        path_iter.next().map(|comp| comp == #s).unwrap_or(false) &&
                    }
                }
                ComponentMatcher::Var(_) => {
                    quote! {
                        path_iter.next().is_some() &&
                    }
                }
            })
            .collect::<TokenStream>();
        quote! {(
            #path.starts_with("/") && {
                let mut path_iter = path[1..].split("/");
                #conditions
                path_iter.next().is_none()
            }
        )}
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
        Ok(Self { components })
    }
}

#[derive(Debug, Clone)]
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

fn is_ident(s: &str) -> bool {
    let s = s.as_bytes();
    s.len() > 0
        && s != b"_"
        && (s[0].is_ascii_alphabetic() || s[0] == b'_')
        && s.iter().all(|&c| c.is_ascii_alphanumeric() || c == b'_')
}
