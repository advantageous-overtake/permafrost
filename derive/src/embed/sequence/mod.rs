mod block;

use block::Block;

use proc_macro2::Delimiter;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::TokenTree as TokenTree2;
use quote::ToTokens as _;
use syn::Ident;
use syn::Token;
use syn::{
    parse::{Parse, ParseStream},
    Result,
};

use super::transform::TransformChain;

/// A single sequence in an embed.
///
/// This can either eb a sequence of tokens or a transformation.
///
/// See [`Sequence::Tokens`] and [`Sequence::Transform`] for more information.
#[derive(Debug, Clone)]
pub enum Sequence {
    /// A single token tree.
    TokenTree(TokenTree2),
    /// A transformed block.
    Transform {
        block: Block,
        /// A [`Block`] finalizer.
        ///
        /// A finalizer is a single transformation that is applied to the entire block.
        ///
        /// # Example
        ///
        /// ```rust, ignore
        /// embed! {
        ///    struct [< ((HELLO)):flatten:case{lower} [world]:ungroup:case{upper} >]:concatenate; // Expands to `helloWORLD
        ///    struct [< ((HELLO)):flatten:case{lower} [world]:ungroup:case{upper} >]:separated{_}; // Expands to `hello_WORLD`
        /// }
        chain: Option<TransformChain>,
    },
}

impl Sequence {
    /// Parse a list of [`Sequence`].
    #[inline]
    pub fn list(input: ParseStream) -> Result<Vec<Self>> {
        let mut sequence_list = Vec::new();

        Ok({
            loop {
                if input.is_empty() {
                    break sequence_list;
                }

                sequence_list.push(input.parse()?);
            }
        })
    }

    /// Expand the sequence into a finished token stream.
    #[inline]
    pub fn expand(self) -> Result<TokenStream2> {
        match self {
            Self::TokenTree(inner) => Ok(inner.into_token_stream()),
            Self::Transform { block, chain } => {
                if let Some(chain) = chain {
                    block.expand().map(|block| chain.expand(block))?
                } else {
                    block.expand()
                }
            }
        }
    }
}

impl Parse for Sequence {
    fn parse(input: ParseStream) -> Result<Self> {
        match input.parse::<TokenTree2>()? {
            TokenTree2::Group(inner) if inner.delimiter() == Delimiter::Bracket => {
                let mut iter = inner.stream().into_iter();

                iter.next()
                    .map(|first| iter.last().map(|last| (first, last)))
                    .flatten()
                    .and_then(|(first, last)| match (first, last) {
                        (TokenTree2::Punct(left), TokenTree2::Punct(right))
                            if left.as_char() == '<' && right.as_char() == '>' =>
                        {
                            Some(syn::parse2::<Block>(inner.stream()).map(|block| {
                                (input.peek(Token![:]) && input.peek2(Ident))
                                    .then(|| input.parse::<TransformChain>())
                                    .transpose()
                                    .map(|chain| Self::Transform { block, chain })
                            }))
                        }
                        _ => None,
                    })
                    .transpose()?
                    .unwrap_or_else(|| Ok(Self::TokenTree(TokenTree2::Group(inner))))
            }
            target_tree @ _ => Ok(Self::TokenTree(target_tree)),
        }
    }
}
