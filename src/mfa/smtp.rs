use super::{
    domain::Otp,
    error::{Error, Result},
    service::MailService,
};
use crate::{on_error, smtp::Smtp, user::domain::Email};
use std::sync::Arc;
use tera::{Context, Tera};

pub struct MfaSmtp<'a, E> {
    pub mta_mailer: Arc<E>,
    pub smtp: &'a Smtp<'a>,
    pub tera: &'a Tera,
    pub otp_subject: &'a str,
    pub otp_template: &'a str,
}

impl<'a, E> MailService for MfaSmtp<'a, E> {
    #[instrument(skip(self))]
    fn send_otp_email(&self, email: &Email, otp: &Otp) -> Result<()> {
        let mut context = Context::new();
        context.insert("name", email.username());
        context.insert("otp", otp.as_ref());

        let body = self
            .tera
            .render(self.otp_template, &context)
            .map_err(on_error!(Error, "rendering otp email template"))?;

        self.smtp
            .send(email, self.otp_subject, &body)
            .map_err(Into::into)
    }
}
