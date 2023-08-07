//! Smtp implementation for sending of predefined email templates.

use crate::base64::B64_CUSTOM_ENGINE;
use crate::result::{Error, Result, StdResult};
use crate::user::application as user_app;
use base64::Engine;
use lettre::address::AddressError;
use lettre::message::{Mailbox, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::Tls;
use lettre::{Message, SmtpTransport, Transport};
use tera::{Context, Tera};

const EMAIL_VERIFICATION_SUBJECT: &str = "Email verification";
const EMAIL_VERIFICATION_TEMPLATE: &str = "verification_email.html";
const EMAIL_RESET_SUBJECT: &str = "Reset password";
const EMAIL_RESET_TEMPLATE: &str = "reset_email.html";

/// Smtp represents an email sender
pub struct Smtp<'a> {
    pub issuer: &'a str,
    pub origin: Mailbox,
    pub verification_subject: &'a str,
    pub verification_template: &'a str,
    pub reset_subject: &'a str,
    pub reset_template: &'a str,
    mailer: SmtpTransport,
    tera: Tera,
}

impl<'a> Smtp<'a> {
    pub fn new(
        origin: &str,
        templates_path: &str,
        smtp_transport: &str,
        smtp_credentials: Option<(String, String)>,
    ) -> StdResult<Self> {
        let origin = origin.parse()?;
        let tera = Tera::new(templates_path)?;

        let transport_attrs: Vec<&str> = smtp_transport.split(':').collect();
        if transport_attrs.is_empty() || transport_attrs[0].is_empty() {
            error!("smtp transport is not valid");
            return Err(Error::Unknown.to_string().into());
        }

        let mut mailer = SmtpTransport::relay(transport_attrs[0])?;

        if transport_attrs.len() > 1 && !transport_attrs[1].is_empty() {
            mailer = mailer.port(transport_attrs[1].parse().unwrap());
        }

        if let Some(credentials) = smtp_credentials {
            let creds = Credentials::new(credentials.0, credentials.1);
            mailer = mailer.credentials(creds);
        } else {
            warn!("transport layer security for smtp disabled");
            mailer = mailer.tls(Tls::None);
        }

        Ok(Smtp {
            issuer: "",
            origin,
            mailer: mailer.build(),
            tera,
            verification_subject: EMAIL_VERIFICATION_SUBJECT,
            verification_template: EMAIL_VERIFICATION_TEMPLATE,
            reset_subject: EMAIL_RESET_SUBJECT,
            reset_template: EMAIL_RESET_TEMPLATE,
        })
    }

    pub fn with_issuer(mut self, issuer: &'a str) -> Self {
        self.issuer = issuer;
        self
    }

    #[instrument(skip(self))]
    fn send_email(&self, to: &str, subject: &str, body: String) -> Result<()> {
        let formated_subject = if !self.issuer.is_empty() {
            format!("[{}] {}", self.issuer, subject)
        } else {
            subject.to_string()
        };

        let to = to.parse().map_err(|err: AddressError| {
            error!(
                to,
                from = self.origin.to_string(),
                error = err.to_string(),
                "parsing verification email destination"
            );
            Error::Unknown
        })?;

        let email = Message::builder()
            .from(self.origin.clone())
            .to(to)
            .subject(formated_subject)
            .singlepart(SinglePart::html(body))
            .map_err(|err| {
                error!(error = err.to_string(), "building email");
                Error::Unknown
            })?;

        self.mailer.send(&email).map_err(|err| {
            error!(error = err.to_string(), "sending email");
            Error::Unknown
        })?;

        Ok(())
    }
}

impl<'a> user_app::Mailer for Smtp<'a> {
    #[instrument(skip(self))]
    fn send_credentials_verification_email(&self, email: &str, token: &str) -> Result<()> {
        let mut context = Context::new();
        context.insert("name", email.split('@').collect::<Vec<&str>>()[0]);
        context.insert("token", &B64_CUSTOM_ENGINE.encode(token));

        let body = self
            .tera
            .render(self.verification_template, &context)
            .map_err(|err| {
                error!(
                    error = err.to_string(),
                    "rendering verification signup email template",
                );
                Error::Unknown
            })?;

        self.send_email(email, self.verification_subject, body)
    }

    #[instrument(skip(self))]
    fn send_credentials_recovery_email(&self, email: &str, token: &str) -> Result<()> {
        let mut context = Context::new();
        context.insert("name", email.split('@').collect::<Vec<&str>>()[0]);
        context.insert("token", &B64_CUSTOM_ENGINE.encode(token));

        let body = self
            .tera
            .render(self.reset_template, &context)
            .map_err(|err| {
                error!(
                    error = err.to_string(),
                    "rendering verification reset email template",
                );
                Error::Unknown
            })?;

        self.send_email(email, self.reset_subject, body)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::result::{Error, Result};
    use crate::user::application::Mailer;

    #[derive(Default)]
    pub struct MailerMock {
        pub force_fail: bool,
    }

    impl Mailer for MailerMock {
        fn send_credentials_verification_email(&self, _: &str, _: &str) -> Result<()> {
            if self.force_fail {
                return Err(Error::Unknown);
            }

            Ok(())
        }

        fn send_credentials_recovery_email(&self, _: &str, _: &str) -> Result<()> {
            if self.force_fail {
                return Err(Error::Unknown);
            }

            Ok(())
        }
    }
}
