use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{token, Field, Fields, FieldsNamed, FieldsUnnamed, Ident};

pub(crate) trait FieldsExt {
    fn try_construct<F, T, E>(&self, ident: &Ident, func: F) -> Result<TokenStream, E>
    where
        F: FnMut(&Field, usize) -> Result<T, E>,
        T: ToTokens;
}

impl FieldsExt for Fields {
    fn try_construct<F, T, E>(&self, ident: &Ident, mut func: F) -> Result<TokenStream, E>
    where
        F: FnMut(&Field, usize) -> Result<T, E>,
        T: ToTokens,
    {
        let mut t = TokenStream::new();
        ident.to_tokens(&mut t);

        match self {
            Fields::Unit => (),
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let mut err = None;
                token::Paren::default().surround(&mut t, |t| {
                    for (i, field) in unnamed.into_iter().enumerate() {
                        match func(field, i) {
                            Ok(x) => x,
                            Err(e) => {
                                err = Some(e);
                                return;
                            }
                        }
                        .to_tokens(t);
                        quote!(,).to_tokens(t);
                    }
                });
                if let Some(e) = err {
                    return Err(e);
                }
            }
            Fields::Named(FieldsNamed { named, .. }) => {
                let mut err = None;
                token::Brace::default().surround(&mut t, |t| {
                    for (i, field) in named.into_iter().enumerate() {
                        field.ident.to_tokens(t);
                        quote!(:).to_tokens(t);
                        match func(field, i) {
                            Ok(x) => x,
                            Err(e) => {
                                err = Some(e);
                                return;
                            }
                        }
                        .to_tokens(t);
                        quote!(,).to_tokens(t);
                    }
                });
                if let Some(e) = err {
                    return Err(e);
                }
            }
        }
        Ok(t)
    }
}
