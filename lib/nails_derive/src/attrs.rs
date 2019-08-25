use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{Attribute, Lit, LitStr, Meta, NestedMeta};

#[cfg(feature = "proc_macro_diagnostics")]
macro_rules! if_proc_macro_diagnostics {
    ($($x:tt)*) => { $($x)* };
}
#[cfg(not(feature = "proc_macro_diagnostics"))]
macro_rules! if_proc_macro_diagnostics {
    ($($x:tt)*) => {};
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StructAttrs {
    pub(crate) path: Option<PathInfo>,
    pub(crate) method: Option<MethodInfo>,
}

impl StructAttrs {
    pub(crate) fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut ret = Self {
            path: None,
            method: None,
        };
        for attr in attrs {
            if !attr.path.is_ident("nails") {
                continue;
            }
            let meta = attr.parse_meta()?;
            let list = match meta {
                Meta::Path(path) => {
                    return Err(syn::Error::new(
                        path.span(),
                        "#[nails] must have an argument list",
                    ));
                }
                Meta::NameValue(nv) => {
                    return Err(syn::Error::new(
                        nv.span(),
                        "#[nails] must have an argument list",
                    ));
                }
                Meta::List(list) => list,
            };
            if_proc_macro_diagnostics! {
                if list.nested.is_empty() {
                    list.paren_token.span.unwrap().warning("#[nails()] is meaningless").emit();
                }
            }
            for item in &list.nested {
                match item {
                    NestedMeta::Meta(meta) => {
                        ret.parse_inner(meta)?;
                    }
                    NestedMeta::Lit(lit) => {
                        return Err(syn::Error::new(lit.span(), "unexpected literal"));
                    }
                }
            }
        }
        Ok(ret)
    }

    fn parse_inner(&mut self, meta: &Meta) -> syn::Result<()> {
        let name = meta.path();
        if name.is_ident("path") {
            self.parse_path(meta)
        } else if name.is_ident("method") {
            self.parse_method(meta)
        } else {
            return Err(syn::Error::new(
                meta.span(),
                format_args!("unknown option: `{}`", path_to_string(name)),
            ));
        }
    }

    fn parse_path(&mut self, meta: &Meta) -> syn::Result<()> {
        let lit = match meta {
            Meta::Path(path) => {
                return Err(syn::Error::new(
                    path.span(),
                    "string value expected in #[nails(path)]",
                ));
            }
            Meta::List(list) => {
                return Err(syn::Error::new(
                    list.paren_token.span,
                    "extra parentheses in #[nails(path)]",
                ));
            }
            Meta::NameValue(nv) => &nv.lit,
        };
        if let Lit::Str(lit) = lit {
            if self.path.is_some() {
                return Err(syn::Error::new(
                    lit.span(),
                    "multiple #[nails(path)] definitions",
                ));
            }
            self.path = Some(PathInfo { path: lit.clone() });
            Ok(())
        } else {
            return Err(syn::Error::new(
                lit.span(),
                "string value expected in #[nails(path)]",
            ));
        }
    }

    fn parse_method(&mut self, meta: &Meta) -> syn::Result<()> {
        let lit = match meta {
            Meta::Path(path) => {
                return Err(syn::Error::new(
                    path.span(),
                    "string value expected in #[nails(method)]",
                ));
            }
            Meta::List(list) => {
                return Err(syn::Error::new(
                    list.paren_token.span,
                    "extra parentheses in #[nails(method)]",
                ));
            }
            Meta::NameValue(nv) => &nv.lit,
        };
        if let Lit::Str(lit) = lit {
            if self.method.is_some() {
                return Err(syn::Error::new(
                    lit.span(),
                    "multiple #[nails(method)] definitions",
                ));
            }
            let lit_str = lit.value();
            let kind = match lit_str.as_str() {
                "GET" => MethodKind::Get,
                "GET_ONLY" => MethodKind::GetOnly,
                "HEAD" => MethodKind::Head,
                "POST" => MethodKind::Post,
                "PUT" => MethodKind::Put,
                "DELETE" => MethodKind::Delete,
                "OPTIONS" => MethodKind::Options,
                "PATCH" => MethodKind::Patch,
                _ => return Err(syn::Error::new(lit.span(), "Unknown method type")),
            };
            self.method = Some(MethodInfo {
                lit: lit.clone(),
                kind,
            });
            Ok(())
        } else {
            return Err(syn::Error::new(
                lit.span(),
                "string value expected in #[nails(path)]",
            ));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PathInfo {
    pub(crate) path: LitStr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MethodInfo {
    pub(crate) lit: LitStr,
    pub(crate) kind: MethodKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MethodKind {
    Get,
    GetOnly,
    Head,
    Post,
    Put,
    Delete,
    Options,
    Patch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FieldAttrs {
    pub(crate) query: Option<QueryFieldInfo>,
    pub(crate) path: Option<PathFieldInfo>,
    pub(crate) body: Option<BodyFieldInfo>,
}

impl FieldAttrs {
    pub(crate) fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut ret = Self {
            query: None,
            path: None,
            body: None,
        };
        for attr in attrs {
            if !attr.path.is_ident("nails") {
                continue;
            }
            let meta = attr.parse_meta()?;
            let list = match meta {
                Meta::Path(path) => {
                    return Err(syn::Error::new(
                        path.span(),
                        "#[nails] must have an argument list",
                    ));
                }
                Meta::NameValue(nv) => {
                    return Err(syn::Error::new(
                        nv.span(),
                        "#[nails] must have an argument list",
                    ));
                }
                Meta::List(list) => list,
            };
            if_proc_macro_diagnostics! {
                if list.nested.is_empty() {
                    list.paren_token.span.unwrap().warning("#[nails()] is meaningless").emit();
                }
            }
            for item in &list.nested {
                match item {
                    NestedMeta::Meta(meta) => {
                        ret.parse_inner(meta)?;
                    }
                    NestedMeta::Lit(lit) => {
                        return Err(syn::Error::new(lit.span(), "unexpected literal"));
                    }
                }
            }
        }
        Ok(ret)
    }

    fn parse_inner(&mut self, meta: &Meta) -> syn::Result<()> {
        let name = meta.path();
        if name.is_ident("query") {
            self.parse_query(meta)
        } else if name.is_ident("path") {
            self.parse_path(meta)
        } else if name.is_ident("body") {
            self.parse_body(meta)
        } else {
            return Err(syn::Error::new(
                meta.span(),
                format_args!("unknown option: `{}`", path_to_string(name)),
            ));
        }
    }

    fn parse_query(&mut self, meta: &Meta) -> syn::Result<()> {
        let (lit, span) = match meta {
            Meta::Path(path) => (None, path.span()),
            Meta::List(list) => {
                return Err(syn::Error::new(
                    list.paren_token.span,
                    "extra parentheses in #[nails(query)]",
                ));
            }
            Meta::NameValue(nv) => {
                if let Lit::Str(lit) = &nv.lit {
                    (Some(lit.clone()), nv.span())
                } else {
                    return Err(syn::Error::new(
                        nv.lit.span(),
                        "string value or no value expected in #[nails(query)]",
                    ));
                }
            }
        };
        if self.query.is_some() {
            return Err(syn::Error::new(
                lit.span(),
                "multiple #[nails(query)] definitions",
            ));
        }
        self.query = Some(QueryFieldInfo { name: lit, span });
        Ok(())
    }

    fn parse_path(&mut self, meta: &Meta) -> syn::Result<()> {
        let (lit, span) = match meta {
            Meta::Path(path) => (None, path.span()),
            Meta::List(list) => {
                return Err(syn::Error::new(
                    list.paren_token.span,
                    "extra parentheses in #[nails(path)]",
                ));
            }
            Meta::NameValue(nv) => {
                if let Lit::Str(lit) = &nv.lit {
                    (Some(lit.clone()), nv.span())
                } else {
                    return Err(syn::Error::new(
                        nv.lit.span(),
                        "string value or no value expected in #[nails(path)]",
                    ));
                }
            }
        };
        if self.path.is_some() {
            return Err(syn::Error::new(
                lit.span(),
                "multiple #[nails(path)] definitions",
            ));
        }
        self.path = Some(PathFieldInfo { name: lit, span });
        Ok(())
    }

    fn parse_body(&mut self, meta: &Meta) -> syn::Result<()> {
        let span = match meta {
            Meta::Path(path) => path.span(),
            Meta::List(list) => {
                return Err(syn::Error::new(
                    list.paren_token.span,
                    "extra parentheses in #[nails(body)]",
                ));
            }
            Meta::NameValue(nv) => {
                return Err(syn::Error::new(
                    nv.lit.span(),
                    "no value expected in #[nails(body)]",
                ));
            }
        };
        if self.body.is_some() {
            return Err(syn::Error::new(span, "multiple #[nails(body)] definitions"));
        }
        self.body = Some(BodyFieldInfo { span });
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct QueryFieldInfo {
    pub(crate) name: Option<LitStr>,
    pub(crate) span: Span,
}

impl PartialEq for QueryFieldInfo {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for QueryFieldInfo {}

#[derive(Debug, Clone)]
pub(crate) struct PathFieldInfo {
    pub(crate) name: Option<LitStr>,
    pub(crate) span: Span,
}

impl PartialEq for PathFieldInfo {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for PathFieldInfo {}

#[derive(Debug, Clone)]
pub(crate) struct BodyFieldInfo {
    pub(crate) span: Span,
}

impl PartialEq for BodyFieldInfo {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
impl Eq for BodyFieldInfo {}

fn path_to_string(path: &syn::Path) -> String {
    use std::fmt::Write;

    let mut s = String::new();
    if path.leading_colon.is_some() {
        s.push_str("::");
    }
    for pair in path.segments.pairs() {
        match pair {
            syn::punctuated::Pair::Punctuated(seg, _) => {
                write!(s, "{}::", seg.ident).ok();
            }
            syn::punctuated::Pair::End(seg) => {
                write!(s, "{}", seg.ident).ok();
            }
        }
    }
    s
}
