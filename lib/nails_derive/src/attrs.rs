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
}

impl StructAttrs {
    pub(crate) fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut ret = Self { path: None };
        for attr in attrs {
            if !attr.path.is_ident("nails") {
                continue;
            }
            let meta = attr.parse_meta()?;
            let list = match meta {
                Meta::Word(ident) => {
                    return Err(syn::Error::new(
                        ident.span(),
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
                    NestedMeta::Literal(lit) => {
                        return Err(syn::Error::new(lit.span(), "unexpected literal"));
                    }
                }
            }
        }
        Ok(ret)
    }

    fn parse_inner(&mut self, meta: &Meta) -> syn::Result<()> {
        let name = meta.name();
        if name == "path" {
            self.parse_path(meta)
        } else {
            return Err(syn::Error::new(
                meta.span(),
                format_args!("unknown option: `{}`", name),
            ));
        }
    }

    fn parse_path(&mut self, meta: &Meta) -> syn::Result<()> {
        let lit = match meta {
            Meta::Word(ident) => {
                return Err(syn::Error::new(
                    ident.span(),
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PathInfo {
    pub(crate) path: LitStr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FieldAttrs {
    pub(crate) query: Option<QueryFieldInfo>,
}

impl FieldAttrs {
    pub(crate) fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut ret = Self { query: None };
        for attr in attrs {
            if !attr.path.is_ident("nails") {
                continue;
            }
            let meta = attr.parse_meta()?;
            let list = match meta {
                Meta::Word(ident) => {
                    return Err(syn::Error::new(
                        ident.span(),
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
                    NestedMeta::Literal(lit) => {
                        return Err(syn::Error::new(lit.span(), "unexpected literal"));
                    }
                }
            }
        }
        Ok(ret)
    }

    fn parse_inner(&mut self, meta: &Meta) -> syn::Result<()> {
        let name = meta.name();
        if name == "query" {
            self.parse_query(meta)
        } else {
            return Err(syn::Error::new(
                meta.span(),
                format_args!("unknown option: `{}`", name),
            ));
        }
    }

    fn parse_query(&mut self, meta: &Meta) -> syn::Result<()> {
        let (lit, span) = match meta {
            Meta::Word(ident) => (None, ident.span()),
            Meta::List(list) => {
                return Err(syn::Error::new(
                    list.paren_token.span,
                    "extra parentheses in #[nails(query)]",
                ));
            }
            Meta::NameValue(nv) => if let Lit::Str(lit) = &nv.lit {
                (Some(lit.clone()), nv.span())
            } else {
                return Err(syn::Error::new(
                    nv.lit.span(),
                    "string value or no value expected in #[nails(query)]",
                ));
            },
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
