mod sequence;
mod transform;


use proc_macro2::Group;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::TokenTree as TokenTree2;
use sequence::Sequence;
use syn::Error;
use syn::{
    parse::{Parse, ParseStream},
    Result,
};

/// An embed is a collection of sequences.
#[derive(Debug, Clone)]
pub struct Embed(Vec<Sequence>);

impl Embed {
    /// Recursively expand the input token stream.
    ///
    /// This visits all [`TokenStream2`]s and expands them, as a requirement for a upper-level expansion.
    #[inline]
    pub fn recursively_expand(input: TokenStream2) -> Result<TokenStream2> {
        let input = input
            .into_iter()
            .try_fold(TokenStream2::new(), |mut acc, target_tree| {
                match target_tree {
                    TokenTree2::Group(group) => {
                        let target_output = Self::recursively_expand(group.stream())?;

                        acc.extend(core::iter::once(TokenTree2::Group(Group::new(
                            group.delimiter(),
                            target_output,
                        ))));
                    }
                    _ => acc.extend(core::iter::once(target_tree)),
                };

                Ok::<_, Error>(acc)
            })?;

        syn::parse2::<Embed>(input).map(Embed::expand)?
    }

    /// Expand the embed into a finished token stream.
    #[inline]
    pub fn expand(self) -> Result<TokenStream2> {
        let Self(inner) = self;

        inner
            .into_iter()
            .try_fold(TokenStream2::new(), |mut stream, sequence| {
                stream.extend(sequence.expand()?);

                Ok(stream)
            })
    }
}

impl Parse for Embed {
    #[inline]
    fn parse(input: ParseStream) -> Result<Self> {
        Sequence::list(input).map(Self)
    }
}
