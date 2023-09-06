use tera::{Context, Tera};

use super::{
    application::Mailer,
    domain::Email,
    error::{Error, Result},
};
use crate::{on_error, smtp::Smtp, token::domain::Token};

pub struct UserSmtp<'a> {
    pub smtp: &'a Smtp<'a>,
    pub tera: &'a Tera,
    pub verification_subject: &'a str,
    pub verification_template: &'a str,
    pub reset_subject: &'a str,
    pub reset_template: &'a str,
}

impl<'a> Mailer for UserSmtp<'a> {
    #[instrument(skip(self))]
    fn send_credentials_verification_email(&self, email: &Email, token: &Token) -> Result<()> {
        let mut context = Context::new();
        context.insert("name", email.as_ref().split('@').collect::<Vec<&str>>()[0]);
        context.insert("token", token.as_ref());

        let body = self
            .tera
            .render(self.verification_template, &context)
            .map_err(on_error!(
                Error,
                "rendering verification signup email template"
            ))?;

        self.smtp.send(email, self.verification_subject, body)
    }

    #[instrument(skip(self))]
    fn send_credentials_reset_email(&self, email: &Email, token: &Token) -> Result<()> {
        let mut context = Context::new();
        context.insert("name", email.as_ref().split('@').collect::<Vec<&str>>()[0]);
        context.insert("token", token.as_ref());

        let body = self
            .tera
            .render(self.reset_template, &context)
            .map_err(on_error!(
                Error,
                "rendering verification reset email template"
            ))?;

        self.smtp.send(email, self.reset_subject, body)
    }
}
