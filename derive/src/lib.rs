#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

pub(crate) mod embed;

use embed::Embed;


/// Embed a sequence of tokens.
/// 
/// This macro allows you to embed and transform arbitrary tokens.
/// 
/// # Example
/// 
/// ```rust, ignore
/// use permafrost_derive::embed;
/// 
/// embed! {
///     [< (hello [world]) >]; // Expands to `(hello [world])`.
///     [< (hello [world]):stringify >]; // Expands to `"(hello [world])"`.
///     [< (hello [world]):ungroup >]; // Expands to `hello [world]`.
/// }
/// ```
#[proc_macro]
pub fn embed(input: TokenStream) -> TokenStream {
    let input = TokenStream2::from(input);

    Embed::recursively_expand(input)
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}
