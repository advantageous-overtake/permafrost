pub use proc_macro2::TokenStream as TokenStream2;
pub use proc_macro2::TokenTree as TokenTree2;
use quote::ToTokens;
use syn::Ident;
use syn::{
    parse::{Parse, ParseStream},
    Token,
};

use super::TransformChain;

/// A segment in a transformation.
///
/// This can either be a modified segment or an untouched segment.
#[derive(Debug, Clone)]
pub enum Segment {
    /// A segment that has been modified.
    ///
    /// This represents a transformation.
    Modified(SegmentModified),
    /// A segment that is untouched.
    ///
    /// This represents a no-op transformation.
    Untouched(SegmentUntouched),
}

/// A modified segment.
///
/// This represents a transformation to a single [`TokenTree2`].
#[derive(Debug, Clone)]
pub struct SegmentModified {
    /// The token tree to apply the transformation to.
    tree: TokenTree2,
    /// The transformation to apply to the token tree.
    chain: TransformChain,
}

/// An untouched [`TokenTree2`].
///
/// This is a no-op transformation.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct SegmentUntouched(TokenTree2);

impl Segment {
    /// Expand the segment into a finished token stream.
    #[inline]
    pub fn expand(self) -> syn::Result<TokenStream2> {
        match self {
            Self::Modified(modified) => modified.expand(),
            Self::Untouched(untouched) => Ok(untouched.0.into_token_stream()),
        }
    }
}

impl Parse for Segment {
    #[inline]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let tree: TokenTree2 = input.parse()?;

        let target_segment = if input.peek(Token![:]) && input.peek2(Ident) {   
            let chain = input.parse()?;

            Self::Modified(SegmentModified { tree, chain })
        } else {
            Self::Untouched(SegmentUntouched(tree))
        };

        Ok(target_segment)
    }
}

impl SegmentModified {
    /// Expand the segment into a finished token stream.
    #[inline]
    pub fn expand(self) -> syn::Result<TokenStream2> {
        let Self { tree, chain } = self;

        chain.expand(tree.into_token_stream())
    }
}
