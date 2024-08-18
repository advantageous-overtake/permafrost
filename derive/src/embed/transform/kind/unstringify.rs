use std::str::FromStr;

use proc_macro2::{TokenStream as TokenStream2, TokenTree as TokenTree2};
use quote::ToTokens;
use syn::{Error, Lit};

use crate::embed::transform::Transformate;

/// A transformation that changes the case of the target [`TokenStream2`].
#[derive(Debug, Clone)]
pub struct TransformUnstringify;

impl Transformate for TransformUnstringify {
    type Args = ();

    fn new(_: TokenStream2) -> Result<Self::Args, Error> {
        Ok(())
    }

    fn apply(input: TokenStream2, _: &Self::Args) -> Result<TokenStream2, Error> {
        input
            .into_iter()
            .try_fold(TokenStream2::new(), |mut acc, target_tree| {
                match target_tree {
                    TokenTree2::Literal(lit) => {
                        match syn::parse2::<Lit>(lit.into_token_stream())? {
                            Lit::Str(target_str) => acc.extend(TokenStream2::from_str(
                                target_str.value().as_str(),
                            )),
                            lit @ _ => acc.extend(lit.into_token_stream()),
                        }
                    }
                    _ => acc.extend(core::iter::once(target_tree)),
                };

                Ok(acc)
            })
    }
}
