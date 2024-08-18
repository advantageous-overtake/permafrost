use std::str::FromStr;

use proc_macro2::{
    Delimiter as Delimiter2, Group as Group2, TokenStream as TokenStream2, TokenTree as TokenTree2,
};
use quote::ToTokens;
use syn::{Ident, Lit};

use convert_case::{Case, Casing};

use crate::embed::transform::Transformate;


/// A transformation that changes the case of the target [`TokenStream2`].
#[derive(Debug, Clone)]
pub struct TransformCase;

impl Transformate for TransformCase {
    type Args = Case;

    fn new(args: TokenStream2) -> Result<Self::Args, syn::Error> {
        let target_ident: Ident = syn::parse2(args)?;

        let target_repr = target_ident.to_string();
        
        // Convert the target ident to a pascal case string, which matches with the `Case` enum variants.
        let target_case = target_repr.to_case(Case::Pascal);

        match Case::all_cases()
            .as_slice()
            .iter()
            .find(|&case| format!("{case:?}") == target_case)
            .copied()
        {
            Some(case) => Ok(case),
            None => Err(syn::Error::new(
                target_ident.span(),
                format!("Unknown case: `{target_repr}`"),
            )),
        }
    }

    fn apply(input: TokenStream2, case: &Self::Args) -> Result<TokenStream2, syn::Error> {
        input
            .into_iter()
            .try_fold(TokenStream2::new(), |mut acc, target_tree| {
                let target_output = match target_tree {
                    TokenTree2::Literal(target_lit) => match syn::parse2::<Lit>(target_lit.into_token_stream())? {
                        Lit::Str(inner) => {
                            TokenStream2::from_str(inner.value().to_case(*case).as_str())?
                        },
                        Lit::Bool(lit) => {
                            TokenStream2::from_str(lit.value.to_string().to_case(*case).as_str())?
                        },

                        lit @ _ => lit.into_token_stream(),
                    },
                    TokenTree2::Ident(target_ident) => {
                        TokenStream2::from_str(target_ident.to_string().to_case(*case).as_str())?
                    }
                    TokenTree2::Group(group) => group
                        .stream()
                        .into_iter()
                        .map(|tree| Self::apply(tree.into_token_stream(), case))
                        .try_fold(TokenStream2::new(), |mut acc, result| {
                            result.map(|stream| {
                                acc.extend(stream);
                                acc
                            })
                        })
                        .map(|a| {
                            let mut new_group = Group2::new(Delimiter2::Brace, a);

                            new_group.set_span(group.span());

                            new_group
                        })
                        .map(TokenTree2::Group)
                        .map(ToTokens::into_token_stream)?,

                    target_tree @ _ => target_tree.into_token_stream(),
                };

                acc.extend(target_output);

                Ok(acc)
            })
    }
}
