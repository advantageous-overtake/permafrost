use proc_macro2::{TokenStream as TokenStream2, TokenTree as TokenTree2};
use quote::ToTokens;

use crate::embed::transform::Transformate;

/// A de-grouping transformation.
///
/// From a [`TokenStream2`] that contains a [`TokenTree2::Group`], this transformation will de-group the contents of the group.
///
/// This is useful for when you want to accept an embedded [`TokenStream2`] atomically, to then pass it to another transformation.
///
///
/// # Example
///
/// ```rust, ignore
/// # use tokel::embed;
///
///
/// macro_rules! unexpected {
///    ($($tt:tt)*) => {
///       embed! {
///         compile_error!([< ($($tt)*):ungroup:stringify >])
///      }
///   };
/// }
///
/// unexpected! {
///   insert racial slur here
/// } // Expands to `compile_error!("insert racial slur here")`.
///
/// ```
#[derive(Debug, Clone)]
pub struct TransformUngroup;

impl Transformate for TransformUngroup {
    type Args = ();

    #[inline]
    fn new(_: TokenStream2) -> Result<Self::Args, syn::Error> {
        Ok(())
    }

    #[inline]
    fn apply(input: TokenStream2, _: &Self::Args) -> Result<TokenStream2, syn::Error> {
        Ok(input
            .into_iter()
            .fold(TokenStream2::new(), |mut acc, target_tree| {
                let target_output = match target_tree {
                    TokenTree2::Group(group) => group.stream(),
                    _ => target_tree.into_token_stream(),
                };

                acc.extend(target_output);

                acc
            }))
    }
}
