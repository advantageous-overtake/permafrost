use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::Error;

use crate::embed::transform::Transformate;

/// A transformation that appends a [`TokenStream2`] to a target [`TokenStream2`].
#[derive(Debug, Clone)]
pub struct TransformAppend;

impl Transformate for TransformAppend {
    type Args = TokenStream2;

    fn new(args: TokenStream2) -> Result<Self::Args, Error> {
        Ok(args)
    }

    fn apply(mut input: TokenStream2, args: &Self::Args) -> Result<TokenStream2, Error> {
        args.to_tokens(&mut input);

        Ok(input)
    }
}
