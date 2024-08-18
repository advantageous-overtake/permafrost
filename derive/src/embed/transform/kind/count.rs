use std::str::FromStr;

use proc_macro2::TokenStream as TokenStream2;

use crate::embed::transform::Transformate;

/// A transformation that counts the amount of [`proc_macro2::TokenTree`] contained in a [`TokenStream2`].
///
/// # Example
///
/// ```rust, ignore
/// # use tokel::embed;
/// embed! {
///     [< (hello [world]):count >]; // Expands to `2`.
/// }
/// ```
#[derive(Debug, Clone)]
pub struct TransformCount;

impl Transformate for TransformCount {
    type Args = ();

    fn new(_: TokenStream2) -> Result<Self::Args, syn::Error> {
        Ok(())
    }

    fn apply(input: TokenStream2, _: &Self::Args) -> Result<TokenStream2, syn::Error> {
        let count = input.into_iter().count();

        TokenStream2::from_str(count.to_string().as_str()).map_err(Into::into)
    }
}
