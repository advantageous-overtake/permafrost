use proc_macro2::TokenStream as TokenStream2;
use syn::Error;

use crate::embed::transform::Transformate;

/// A transformation that prefixes a [`TokenStream2`] to a target [`TokenStream2`].
#[derive(Debug, Clone)]
pub struct TransformPrefix;

impl Transformate for TransformPrefix {
    type Args = TokenStream2;

    fn new(args: TokenStream2) -> Result<Self::Args, Error> {
        Ok(args)
    }

    fn apply(input: TokenStream2, args: &Self::Args) -> Result<TokenStream2, Error> {
        Ok(
            args.clone()
                .into_iter()
                .chain(input.into_iter())
                .collect::<TokenStream2>(),
        )
    }
}
