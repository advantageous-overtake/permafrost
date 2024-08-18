use proc_macro2::TokenStream as TokenStream2;
use syn::{ parse::{Parse, ParseStream}, Result, Token};

use crate::embed::transform::Segment;


/// A singular transformation block.
///
/// This is a collection of segments, each of which can be either modified or untouched.
///
/// See [`Segment`] for on how [`Block`] is constructed.
#[derive(Debug, Clone)]
pub struct Block {
    /// A list of segments.
    /// 
    /// See [`Segment`].
    segment_list: Vec<Segment>,
}

impl Block {
    /// Expand the transformation into a finished token stream.
    #[inline]
    pub fn expand(self) -> Result<TokenStream2> {
        let Self { segment_list, .. } = self;

        segment_list
            .into_iter()
            .try_fold(TokenStream2::new(), |mut stream, segment| {
                stream.extend(segment.expand()?);

                Ok(stream)
            })
    }
}

impl Parse for Block {
    #[inline]
    fn parse(input: ParseStream) -> Result<Self> {
        let mut segment_list = Vec::new();

        input.parse::<Token![<]>()?;

        Ok(loop {
            if input.peek(Token![>]) {
                input.parse::<Token![>]>()?;

                break Block { segment_list };
            }

            segment_list.push(input.parse()?);
        })
    }
}
