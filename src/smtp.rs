//! Smtp implementation for sending of predefined email templates.

use crate::on_error;
use crate::user::domain::Email;
use lettre::message::{Mailbox, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::Tls;
use lettre::{Message, SmtpTransport, Transport};
use once_cell::sync::Lazy;
use std::env;

const DEFAULT_TEMPLATES_PATH: &str = "/etc/rauth/smtp/templates/*.html";
const DEFAULT_VERIFICATION_SUBJECT: &str = "Email verification";
const DEFAULT_VERIFICATION_TEMPLATE: &str = "verification_email.html";
const DEFAULT_RESET_SUBJECT: &str = "Reset password";
const DEFAULT_RESET_TEMPLATE: &str = "reset_email.html";

const ENV_SMTP_TRANSPORT: &str = "SMTP_TRANSPORT";
const ENV_SMTP_USERNAME: &str = "SMTP_USERNAME";
const ENV_SMTP_PASSWORD: &str = "SMTP_PASSWORD";
const ENV_SMTP_ISSUER: &str = "SMTP_ISSUER";
const ENV_SMTP_TEMPLATES: &str = "SMTP_TEMPLATES";
const ENV_SMTP_ORIGIN: &str = "SMTP_ORIGIN";

pub static SMTP_TRANSPORT: Lazy<String> =
    Lazy::new(|| env::var(ENV_SMTP_TRANSPORT).expect("smtp transport must be set"));

pub static SMTP_USERNAME: Lazy<String> =
    Lazy::new(|| env::var(ENV_SMTP_USERNAME).unwrap_or_default());

pub static SMTP_PASSWORD: Lazy<String> =
    Lazy::new(|| env::var(ENV_SMTP_PASSWORD).unwrap_or_default());

pub static SMTP_ORIGIN: Lazy<String> =
    Lazy::new(|| env::var(ENV_SMTP_ORIGIN).expect("smpt origin must be set"));

pub static SMTP_ISSUER: Lazy<String> =
    Lazy::new(|| env::var(ENV_SMTP_ISSUER).expect("smtp issuer must be set"));

pub static SMTP_TEMPLATES: Lazy<String> = Lazy::new(|| {
    env::var(ENV_SMTP_TEMPLATES).unwrap_or_else(|_| DEFAULT_TEMPLATES_PATH.to_string())
});

/// A builder for the [Smtp] struct.
pub struct SmtpBuilder<'a> {
    pub issuer: &'a str,
    pub origin: &'a str,
    pub templates: &'a str,
    pub transport: &'a str,
    pub username: &'a str,
    pub password: &'a str,
}

impl<'a> Default for SmtpBuilder<'a> {
    fn default() -> Self {
        Self {
            issuer: "",
            origin: "",
            templates: "",
            transport: "",
            username: "",
            password: "",
        }
    }
}

impl<'a> SmtpBuilder<'a> {
    pub fn build(&self) -> Result<Smtp<'a>, Box<dyn std::error::Error>> {
        let transport_attrs: Vec<&str> = self.transport.split(':').collect();
        if transport_attrs.is_empty() || transport_attrs[0].is_empty() {
            return Err("smtp transport is not valid".into());
        }

        let mut mailer = SmtpTransport::relay(transport_attrs[0])?;

        if transport_attrs.len() > 1 && !transport_attrs[1].is_empty() {
            mailer = mailer.port(transport_attrs[1].parse().unwrap());
        }

        if !self.username.is_empty() && !self.password.is_empty() {
            let creds = Credentials::new(self.username.to_string(), self.password.to_string());
            mailer = mailer.credentials(creds);
        } else {
            warn!("tls is disabled for smtp");
            mailer = mailer.tls(Tls::None);
        }

        Ok(Smtp {
            issuer: self.issuer,
            origin: self.origin.parse()?,
            mailer: mailer.build(),
        })
    }
}

// TODO: make build a generic and 'static' form of SMT in order to build several mailers.
/// Smtp represents an email sender.
pub struct Smtp<'a> {
    pub issuer: &'a str,
    pub origin: Mailbox,
    pub mailer: SmtpTransport,
}

impl<'a> Smtp<'a> {
    #[instrument(skip(self))]
    pub fn send<Err>(&self, to: &Email, subject: &str, body: String) -> Result<(), Err>
    where
        Err: From<String>,
    {
        let formated_subject = self
            .issuer
            .is_empty()
            .then_some(subject.to_string())
            .unwrap_or_else(|| format!("[{}] {}", self.issuer, subject));

        let to = to
            .as_ref()
            .parse()
            .map_err(on_error!("parsing email destination"))?;

        let email = Message::builder()
            .from(self.origin.clone())
            .to(to)
            .subject(formated_subject)
            .singlepart(SinglePart::html(body))
            .map_err(on_error!("building email message"))?;

        self.mailer
            .send(&email)
            .map_err(on_error!("sending email"))?;

        Ok(())
    }
}
