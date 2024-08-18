use proc_macro2::TokenStream as TokenStream2;

use crate::embed::transform::Transformate;

/// A transformation that reverses the target [`TokenStream2`].
///
/// This transformation is useful for when you want to reverse the order of the target [`TokenStream2`].
///
/// # Example
///
/// ```rust, ignore
/// # use tokel::embed;
/// embed! {
///     [< (hello [world]):reverse >]; // Expands to `[world] hello`.
/// }
/// ```
#[derive(Debug, Clone)]
pub struct TransformReverse;

impl Transformate for TransformReverse {
    type Args = ();

    fn new(_: TokenStream2) -> Result<Self::Args, syn::Error> {
        Ok(())
    }

    fn apply(input: TokenStream2, _: &Self::Args) -> Result<TokenStream2, syn::Error> {
        let mut tokens = input.into_iter().collect::<Vec<_>>();

        tokens.reverse();

        Ok(tokens.into_iter().collect())
    }
}
