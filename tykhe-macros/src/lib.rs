extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use std::ops::Not;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, ItemFn,
};

/// Decorates a function to verify and extact information from its associated token argument.
#[proc_macro_attribute]
pub fn with_token(metadata: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(metadata as WithTokenArgs);
    let Some(token_kind) = args.token_kind else {
        return quote!(compile_error!("the `kind` argument has to be set");).into();
    };

    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = parse_macro_input!(input as ItemFn);

    let stmts = &block.stmts;

    let resolve_user_id = args.no_user_id.not().then(|| {
        quote! {
           let user_id = UserID::from_str(claims.payload().subject()).map_err(on_error!(
               uuid::Error as Error,
               "parsing token subject into user id"
           ))?;
        }
    });

    TokenStream::from(quote! {
        #(#attrs)* #vis #sig {
            let claims = self.token_srv.claims(token).await?;
            if !matches!(claims.payload().kind(), TokenKind::#token_kind) {
                return Err(Error::WrongToken);
            }

            #resolve_user_id

            #(#stmts)*
        }
    })
}

#[derive(Default)]
struct WithTokenArgs {
    /// TokenKind with which the token has to match.
    token_kind: Option<Ident>,
    /// Skip user ID processing. No user_id variable will be available in scope.
    no_user_id: bool,
}

impl Parse for WithTokenArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut args = Self::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::kind) {
                if args.token_kind.is_some() {
                    return Err(input.error("expected only a single `kind` argument"));
                }

                let TokenKind(token_kind) = input.parse()?;
                args.token_kind = Some(token_kind);
            } else if lookahead.peek(kw::no_user_id) {
                let _ = input.parse::<kw::no_user_id>()?;
                args.no_user_id = true;
            } else {
                // Parse the unrecognized token tree to advance the parse
                // stream, and throw it away so we can keep parsing. Otherwise
                // this would become an endless while loop.
                let _ = input.parse::<proc_macro2::TokenTree>();
            }
        }

        Ok(args)
    }
}

struct TokenKind(Ident);

impl Parse for TokenKind {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _ = input.parse::<kw::kind>();
        let content;
        let _ = syn::parenthesized!(content in input);

        Ok(Self(content.parse()?))
    }
}

mod kw {
    syn::custom_keyword!(kind);
    syn::custom_keyword!(no_user_id);
}
