use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{token, Field, Fields, FieldsNamed, FieldsUnnamed};
use synstructure::VariantInfo;

pub(crate) trait VariantInfoExt {
    fn try_construct<F, T, E>(&self, func: F) -> Result<TokenStream, E>
    where
        F: FnMut(&Field, usize) -> Result<T, E>,
        T: ToTokens;
}

impl VariantInfoExt for VariantInfo<'_> {
    fn try_construct<F, T, E>(&self, mut func: F) -> Result<TokenStream, E>
    where
        F: FnMut(&Field, usize) -> Result<T, E>,
        T: ToTokens,
    {
        let mut t = TokenStream::new();
        if let Some(prefix) = self.prefix {
            quote!(#prefix ::).to_tokens(&mut t);
        }
        self.ast().ident.to_tokens(&mut t);

        match *self.ast().fields {
            Fields::Unit => (),
            Fields::Unnamed(FieldsUnnamed { ref unnamed, .. }) => {
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
            Fields::Named(FieldsNamed { ref named, .. }) => {
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
