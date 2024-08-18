use std::error::Error;

use proc_macro2::TokenStream as TokenStream2;

/// A trait for individual transformations.
/// 
/// A [`Transformate`] is a possibly-recursive transformation, that can be applied to a [`TokenStream2`].
/// 
/// These are the building blocks for this crate, and are used to transform the input token tree.
/// 
/// These are stricly pure and do not have any side-effects.
pub trait Transformate<E = syn::Error>
where
    E: Error,
{
    /// The arguments required for this transformation.
    /// 
    /// This is not left to the implementor to decide as the transformation can be unsized.
    type Args;

    /// Parse the arguments required for this transformation.
    /// 
    /// See [`Transformate::Args`] for more.
    fn new(args: TokenStream2) -> Result<Self::Args, E>;

    /// Apply the transformation to the target stream, with the given arguments.
    fn apply(input: TokenStream2, args: &Self::Args) -> Result<TokenStream2, E>;
}