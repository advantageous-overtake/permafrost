use std::str::FromStr as _;

use proc_macro2::TokenStream as TokenStream2;

use crate::embed::transform::Transformate;

/// A transformation that stringifies the target [`TokenStream2`].
///
/// This transformation is useful for when you want to convert the target [`TokenStream2`] into a string.
///
///
/// # Example
///
/// ```rust, ignore
///
/// # use tokel::embed;
///
/// embed! {
///    [< (hello [world]):stringify >]; // Expands to `"(hello [world])"`.
///    [< (hello [world]):ungroup:stringify >]; // Expands to `"hello [world]"`.
/// }
#[derive(Debug, Clone)]
pub struct TransformStringify;

/// The selected mode for stringification.
#[derive(Default)]
pub enum Mode {
    /// The default mode.
    ///
    /// This implies basic stringification, without additional computation.
    #[default]
    Unspecified,
}

impl Transformate for TransformStringify {
    type Args = Mode;

    fn new(_: TokenStream2) -> Result<Self::Args, syn::Error> {
        Ok(Mode::default())
    }

    fn apply(input: TokenStream2, _: &Self::Args) -> Result<TokenStream2, syn::Error> {
        let target_output = dbg!(input.to_string());

        let mut separator = String::from('#');

        while target_output.contains(separator.as_str()) {
            separator.push('#');
        }

        TokenStream2::from_str(&format!("r{separator}\"{target_output}\"{separator}"))
            .map_err(Into::into)
    }
}
