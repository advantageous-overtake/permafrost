mod append;
mod case;
mod concatenate;
mod flatten;
mod prefix;
mod reverse;
mod stringify;
mod ungroup;
mod unstringify;
mod count;

pub use self::{
    case::TransformCase, concatenate::TransformConcatenate, flatten::TransformFlatten,
    reverse::TransformReverse, stringify::TransformStringify, ungroup::TransformUngroup,
};

use append::TransformAppend;
use count::TransformCount;
use prefix::TransformPrefix;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::TokenTree as TokenTree2;
use syn::Ident;
use syn::{
    parse::{Parse, ParseStream},
    token::Brace,
    Error, Token,
};

use phf::{phf_map, Map};
use unstringify::TransformUnstringify;

use super::Transformate;

static RECOGNIZED_TRANSFORMS: Map<&str, TransformKind> = phf_map! {
    "case" => TransformKind::Case,
    "ungroup" => TransformKind::Ungroup,
    "flatten" => TransformKind::Flatten,
    "reverse" => TransformKind::Reverse,
    "stringify" => TransformKind::Stringify,
    "unstringify" => TransformKind::Unstringify,
    "concatenate" => TransformKind::Concatenate,
    "append" => TransformKind::Append,
    "prefix" => TransformKind::Prefix,
    "count" => TransformKind::Count,
};

/// A single transformation kind.
///
/// This encapsulates a single transformation, without its target token tree.
#[derive(Debug, Clone)]
#[repr(u8)]
pub enum TransformKind {
    /// Convert the token tree to a specific case.
    ///
    /// Using the [`Case`] enum, this will convert the token tree to the specified case.
    ///
    /// # Example
    ///
    /// ```rust, ignore
    /// # use tokel::embed;
    /// embed! {
    ///    struct [< hello:case{pascal} >]; // Expands to `struct Hello;`.
    /// }
    /// ```
    Case,
    /// Flatten the [`TokenTree2`] if it is a [`TokenTree2::Group`], it is a no-op otherwise.
    ///
    /// This will remove all grouping tokens from the token tree.
    ///
    /// # Example
    ///
    /// ```rust, ignore
    /// # use tokel::embed;
    /// embed! {
    ///  [< (hello [world]):ungroup >]; // Expands to `hello world`.
    /// }
    Ungroup,

    /// Recursively flatten the token tree.
    ///
    /// This will remove all grouping tokens from the token tree.
    ///
    /// # Example
    ///
    /// ```rust, ignore
    /// # use tokel::embed;
    /// embed! {
    ///   [< (hello [world]):flatten >]; // Expands to `hello world`.
    /// }
    Flatten,

    /// Reverse the [`TokenTree2`].
    ///
    /// Reverses the inner [`TokenStream2`] in case of [`TokenTree2::Group`], leaves it untouched otherwise.
    ///
    /// # Example
    ///
    /// ```rust, ignore
    /// # use tokel::embed;
    /// embed! {
    ///  [< (hello [world]):reverse >]; // Expands to `[world] hello`.
    /// }
    Reverse,

    /// Turn the [`TokenTree2`] into a string literal.
    Stringify,

    /// Turn any string literals in the [`TokenTree2`] into a [`TokenStream2`].
    ///
    /// This is not necessarily the inverse of [`TransformKind::Stringify`], albeit it may be for some input-output pairs.
    Unstringify,

    /// Concatenate the [`TokenTree2`] into a single token, consicutively flatting any groups.
    ///
    /// [`TransformKind::Concatenate`] operates in two modes:
    ///
    /// - `Concatenate::Ident` will concatenate the tokens into a single identifier (`default mode`).
    /// - `Concatenate::String`
    ///
    /// # Example
    ///
    /// ```rust, ignore
    /// # use tokel::embed;
    /// embed! {
    ///  [< (hello [world]):concatenate >]; // Expands to `helloworld`.
    /// }
    Concatenate,

    /// Append a [`TokenStream2`] to the provided input.
    Append,

    /// Prefix a [`TokenStream2`] to the provided input.
    Prefix,

    /// Count the number of token trees in the [`TokenStream2`].
    Count,
    // TODO: Add more transformations.
    //
    // For example:
    // - `Shuffle` to shuffle the tokens in a token tree.
    // - `Sort` to sort the tokens in a token tree lexico-graphically. (might want to do this on multiple criteria).
    // - `Reverse` to reverse the tokens in a token tree. (can be done with macros-by-example but is recursion heavy).
    // - `Unique` to remove duplicate tokens in a token tree.
}

/// Represents a pending transformation.
///
/// This is used to represent a transformation that has not yet been applied to a [`TokenStream2`].
#[derive(Debug, Clone)]
pub struct Transform {
    pub kind: TransformKind,
    pub args: TokenStream2,
}

/// A transformation chain.
///
/// Multiple transformations can be chained together to form a transformation tree,
/// this allows for powerful high-level transformations to be applied to a token tree.
///
/// See the [`TransformKind`] enum for the available transformations.
#[derive(Debug, Clone)]
pub enum TransformChain {
    /// A terminal transformation.
    End(Transform),
    /// A non-terminal transformation.
    Next(Transform, Box<Self>),
}

impl Parse for TransformKind {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![:]>()?;

        let kind = input.parse::<syn::Ident>()?;

        RECOGNIZED_TRANSFORMS
            .get(kind.to_string().as_str())
            .cloned()
            .ok_or_else(|| {
                let available_transforms = RECOGNIZED_TRANSFORMS.keys().map(|key| format!("`{}`", key)).collect::<Vec<_>>().join(" ");

                Error::new(kind.span(), format!("unrecognized transform: `{}`\navailable transforms are: {available_transforms}", kind))

            })
    }
}

impl IntoIterator for TransformChain {
    type Item = Transform;
    type IntoIter = <Vec<Transform> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        /// Recursively append [`Transform`]s to the provided [`Vec`].
        fn append_to(transforms: &mut Vec<Transform>, chain: TransformChain) {
            match chain {
                TransformChain::End(transform) => transforms.push(transform),
                TransformChain::Next(transform, next) => {
                    transforms.push(transform);

                    // TODO: use `become` expression when stable
                    append_to(transforms, *next);
                }
            }
        }

        let mut transforms = Vec::new();

        append_to(&mut transforms, self);

        transforms.into_iter()
    }
}

impl TransformChain {
    /// Expand the transformation chain into a finished token stream.
    #[inline]
    pub fn expand(self, tree: TokenStream2) -> syn::Result<TokenStream2> {
        self.into_iter()
            .try_fold(tree, |acc, Transform { kind, args }| match kind {
                TransformKind::Case => <TransformCase as Transformate>::new(args)
                    .and_then(|args| TransformCase::apply(acc, &args)),
                TransformKind::Flatten => <TransformFlatten as Transformate>::new(args)
                    .and_then(|args| TransformFlatten::apply(acc, &args)),
                TransformKind::Ungroup => <TransformUngroup as Transformate>::new(args)
                    .and_then(|args| TransformUngroup::apply(acc, &args)),
                TransformKind::Stringify => <TransformStringify as Transformate>::new(args)
                    .and_then(|args| TransformStringify::apply(acc, &args)),
                TransformKind::Reverse => <TransformReverse as Transformate>::new(args)
                    .and_then(|args| TransformReverse::apply(acc, &args)),
                TransformKind::Append => <TransformAppend as Transformate>::new(args)
                    .and_then(|args| TransformAppend::apply(acc, &args)),
                TransformKind::Prefix => <TransformPrefix as Transformate>::new(args)
                    .and_then(|args| TransformPrefix::apply(acc, &args)),
                TransformKind::Concatenate => <TransformConcatenate as Transformate>::new(args)
                    .and_then(|args| TransformConcatenate::apply(acc, &args)),
                TransformKind::Unstringify => <TransformUnstringify as Transformate>::new(args)
                    .and_then(|args| TransformUnstringify::apply(acc, &args)),
                TransformKind::Count => <TransformCount as Transformate>::new(args)
                    .and_then(|args| TransformCount::apply(acc, &args)),   
            })
    }
}

impl Parse for TransformChain {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let kind = input.parse::<TransformKind>()?;

        let args = input
            .peek(Brace)
            .then(|| input.parse::<TokenTree2>())
            .transpose()?
            .map(|tree| match tree {
                TokenTree2::Group(group) => group.stream(),
                _ => unreachable!("peeked for a brace, but did not find a group"),
            })
            .unwrap_or_default();

        if input.peek(Token![:]) && input.peek2(Ident) {
            Ok(TransformChain::Next(
                Transform { kind, args },
                Box::new(input.parse::<TransformChain>()?),
            ))
        } else {
            Ok(TransformChain::End(Transform { kind, args }))
        }
    }
}
