//! Implementation details for the `uuid!` macro.
//!
//! This crate is not meant to be used directly. Instead,
//! you can use the `macro-diagnostics` feature of `uuid`:
//!
//! ```toml
//! [dependencies.uuid]
//! features = ["macro-diagnostics"]
//! ```

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use std::fmt;
use syn::{LitStr, spanned::Spanned};

mod error;
mod parser;

#[proc_macro]
#[doc(hidden)]
pub fn parse_lit(input: TokenStream) -> TokenStream {
    build_uuid(input.clone()).unwrap_or_else(|e| {
        let msg = e.to_string();
        let span =
            match e {
                Error::UuidParse(lit, error::Error(error::ErrorKind::Char {
                    character,
                    index,
                })) => {
                    let mut bytes = character as u32;
                    let mut width = 0;
                    while bytes != 0 {
                        bytes >>= 4;
                        width += 1;
                    }
                    let mut s = proc_macro2::Literal::string("");
                    s.set_span(lit.span());
                    s.subspan(index..index + width - 1)
                }
                Error::UuidParse(lit, error::Error(
                    error::ErrorKind::GroupLength { index, len, .. },
                )) => {
                    let mut s = proc_macro2::Literal::string("");
                    s.set_span(lit.span());
                    s.subspan(index..index + len)
                }
                _ => None,
            }
            .unwrap_or_else(|| TokenStream2::from(input).span());

        TokenStream::from(quote_spanned! {span=>
            compile_error!(#msg)
        })
    })
}

enum Error {
    NonStringLiteral,
    UuidParse(LitStr, error::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::NonStringLiteral => f.write_str("expected string literal"),
            Error::UuidParse(_, ref e) => write!(f, "{}", e),
        }
    }
}

fn build_uuid(input: TokenStream) -> Result<TokenStream, Error> {
    let str_lit = match syn::parse::<syn::Lit>(input) {
        Ok(syn::Lit::Str(literal)) => literal,
        _ => return Err(Error::NonStringLiteral),
    };

    let bytes = parser::try_parse(&str_lit.value())
        .map_err(|e| Error::UuidParse(str_lit, e.into_err()))?;

    let tokens = bytes
        .iter()
        .map(|byte| quote! { #byte, })
        .collect::<TokenStream2>();

    Ok(quote! {[#tokens]}.into())
}
