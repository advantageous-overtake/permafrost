use std::str::FromStr as _;

use phf::{phf_map, Map};
use proc_macro2::{TokenStream as TokenStream2, TokenTree as TokenTree2};
use quote::ToTokens;
use syn::{spanned::Spanned, Error, Ident, Lit};

use crate::embed::transform::Transformate;

use super::TransformFlatten;

static RECOGNIZED_MODES: Map<&str, Concatenate> = phf_map! {
    "ident" => Concatenate::Ident,
    "r#ident" => Concatenate::RawIdent,
    "string" => Concatenate::String,
};

/// A transformation that changes the case of the target [`TokenStream2`].
#[derive(Debug, Clone)]
pub struct TransformConcatenate;

/// The selected mode for concatenation.
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Concatenate {
    /// Attempt to convert everything to an identifier.
    #[default]
    Ident,

    /// Attempt to convert everything to a raw identifier.
    RawIdent,

    /// Attempt to convert everything to a string.
    String,
}

impl Transformate for TransformConcatenate {
    type Args = Concatenate;

    fn new(args: TokenStream2) -> Result<Self::Args, Error> {
        if args.is_empty() {
            return Ok(Concatenate::Ident);
        }

        let target_ident: Ident = syn::parse2(args)?;

        let target_repr = target_ident.to_string();

        RECOGNIZED_MODES
            .get(target_repr.as_str())
            .copied()
            .ok_or_else(|| {
                let target_span = target_ident.span();

                Error::new(
                    target_span,
                    format!(
                        "unknown mode: `{target_repr}`, valid modes are: {modes}",
                        modes = RECOGNIZED_MODES
                            .keys()
                            .copied()
                            .collect::<Vec<&str>>()
                            .join(" ")
                    ),
                )
            })
    }

    fn apply(input: TokenStream2, args: &Self::Args) -> Result<TokenStream2, Error> {
        match args {
            Concatenate::Ident | Concatenate::RawIdent => {
                let span = input.span();

                let input = <TransformFlatten as Transformate>::apply(input, &())?;

                let mut target_ident = String::new();

                for target_tree in input {
                    match target_tree {
                        TokenTree2::Punct(punct) => target_ident.push(punct.as_char()),
                        TokenTree2::Literal(lit) => {
                            match syn::parse2::<Lit>(lit.into_token_stream())? {
                                Lit::Str(target_str) => target_ident.push_str(&target_str.value()),
                                Lit::Char(target_char) => target_ident.push(target_char.value()),
                                Lit::Int(target_int) => target_ident
                                    // NOTE: This gets rid of the literal suffix.
                                    .push_str(&target_int.base10_digits()),
                                Lit::Float(target_float) => target_ident
                                    // NOTE: This gets rid of the literal suffix.
                                    .push_str(&target_float.base10_digits()),
                                Lit::Bool(target_bool) => {
                                    target_ident.push_str(&target_bool.value.to_string())
                                }
                                Lit::Verbatim(lit) => target_ident.push_str(&lit.to_string()),
                                lit @ (Lit::Byte(_) | Lit::ByteStr(_)) => {
                                    return Err(Error::new(
                                        lit.span(),
                                        "byte literals are not supported in concatenation",
                                    ))
                                }
                                _ => unimplemented!(),
                            }
                        }
                        TokenTree2::Ident(ident) => target_ident.push_str(&ident.to_string()),
                        TokenTree2::Group(_) => unreachable!("found group in flattened stream"),
                    }
                }

                let target_ident = if *args == Concatenate::RawIdent {
                    Ident::new(format!("r#{}", target_ident).as_str(), span)
                } else {
                    Ident::new(target_ident.as_str(), span)
                };

                Ok(target_ident.into_token_stream())
            }
            Concatenate::String => {
                let output =
                    input
                        .into_iter()
                        .try_fold(String::new(), |mut acc, target_tree| {
                            let output = match target_tree {
                                TokenTree2::Ident(ident) => ident.to_string(),
                                TokenTree2::Punct(punct) => punct.to_string(),
                                TokenTree2::Literal(lit) => {
                                    match syn::parse2::<syn::Lit>(lit.into_token_stream())? {
                                        syn::Lit::Str(target_str) => target_str.value(),
                                        lit @ _ => lit.into_token_stream().to_string(),
                                    }
                                }
                                _ => target_tree.to_string(),
                            };

                            acc.push_str(&output);

                            Ok::<_, Error>(acc)
                        })?;

                let mut separator = String::from('#');

                while output.contains(separator.as_str()) {
                    separator.push('#');
                }

                TokenStream2::from_str(&format!("r{separator}\"{output}\"{separator}"))
                    .map_err(Into::into)
            }
        }
    }
}
