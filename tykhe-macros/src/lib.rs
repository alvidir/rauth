extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    parse_macro_input, Error, Ident, ItemFn, Result, TypeParam,
};

#[derive(Default)]
struct WithTokenArgs {
    token_kind: Option<TypeParam>,
}

impl Parse for WithTokenArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut args = Self::default();
        if input.is_empty() {
            return Err(Error::new(input.span(), "expected a token kind argument"));
        }

        let lookahead = input.lookahead1();
        if !lookahead.peek(Ident::peek_any) {
            return Err(Error::new(input.span(), "unexpected argument"));
        }

        args.token_kind = Some(input.parse()?);
        Ok(args)
    }
}

#[proc_macro_attribute]
pub fn with_token(metadata: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(metadata as WithTokenArgs);
    let Some(token_kind) = args.token_kind else {
        panic!("expected a token kind argument")
    };

    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = parse_macro_input!(input as ItemFn);

    let stmts = &block.stmts;

    TokenStream::from(quote! {
        #(#attrs)* #vis #sig {
            let claims = self.token_srv.claims(token).await?;
            if !matches!(claims.payload().kind(), TokenKind::#token_kind) {
                return Err(Error::WrongToken);
            }

            let user_id = UserID::from_str(claims.payload().subject()).map_err(on_error!(
                uuid::Error as Error,
                "parsing token subject into user id"
            ))?;

            #(#stmts)*
        }
    })
}

mod kw {}
