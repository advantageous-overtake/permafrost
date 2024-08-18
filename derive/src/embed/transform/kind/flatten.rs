use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::TokenTree as TokenTree2;
use quote::ToTokens as _;

use crate::embed::transform::Transformate;

/// A transformation that flattens the target [`TokenStream2`].
///
/// # Example
///
/// ```rust, ignore
/// # use tokel::embed;
/// embed! {
///     [< (hello [world]):flatten >]; // Expands to `hello world`.
/// }
/// ```
#[derive(Debug, Clone)]
pub struct TransformFlatten;

impl Transformate for TransformFlatten {
    type Args = ();

    fn new(_: TokenStream2) -> Result<Self::Args, syn::Error> {
        Ok(())
    }

    fn apply(input: TokenStream2, args: &Self::Args) -> Result<TokenStream2, syn::Error> {
        input
            .into_iter()
            .try_fold(TokenStream2::new(), |mut acc, target_tree| {
                let target_output = match target_tree {
                    TokenTree2::Group(group) => <Self as Transformate>::apply(group.stream(), args)?,
                    _ => target_tree.into_token_stream(),
                };

                target_output.to_tokens(&mut acc);

                Ok(acc)
            })
    }
}
