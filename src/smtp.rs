//! Smtp implementation for sending of predefined email templates.

use crate::base64::B64_CUSTOM_ENGINE;
use crate::result::{Error, Result, StdResult};
use crate::user::application as user_app;
use crate::user::domain::Email;
use base64::Engine;
use lettre::address::AddressError;
use lettre::message::{Mailbox, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::Tls;
use lettre::{Message, SmtpTransport, Transport};
use once_cell::sync::Lazy;
use std::env;
use tera::{Context, Tera};

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
    pub verification_subject: &'a str,
    pub verification_template: &'a str,
    pub reset_subject: &'a str,
    pub reset_template: &'a str,
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
            verification_subject: DEFAULT_VERIFICATION_SUBJECT,
            verification_template: DEFAULT_VERIFICATION_TEMPLATE,
            reset_subject: DEFAULT_RESET_SUBJECT,
            reset_template: DEFAULT_RESET_TEMPLATE,
        }
    }
}

impl<'a> SmtpBuilder<'a> {
    pub fn build(&self) -> StdResult<Smtp<'a>> {
        let transport_attrs: Vec<&str> = self.transport.split(':').collect();
        if transport_attrs.is_empty() || transport_attrs[0].is_empty() {
            error!("smtp transport is not valid");
            return Err(Error::Unknown.to_string().into());
        }

        let mut mailer = SmtpTransport::relay(transport_attrs[0])?;

        if transport_attrs.len() > 1 && !transport_attrs[1].is_empty() {
            mailer = mailer.port(transport_attrs[1].parse().unwrap());
        }

        if !self.username.is_empty() && !self.password.is_empty() {
            let creds = Credentials::new(self.username.to_string(), self.password.to_string());
            mailer = mailer.credentials(creds);
        } else {
            warn!("transport layer security for smtp disabled");
            mailer = mailer.tls(Tls::None);
        }

        Ok(Smtp {
            issuer: self.issuer,
            origin: self.origin.parse()?,
            mailer: mailer.build(),
            tera: Tera::new(self.templates)?,
            verification_subject: self.verification_subject,
            verification_template: self.verification_template,
            reset_subject: self.reset_subject,
            reset_template: self.reset_template,
        })
    }
}

/// Smtp represents an email sender.
pub struct Smtp<'a> {
    pub issuer: &'a str,
    pub origin: Mailbox,
    pub verification_subject: &'a str,
    pub verification_template: &'a str,
    pub reset_subject: &'a str,
    pub reset_template: &'a str,
    pub mailer: SmtpTransport,
    pub tera: Tera,
}

impl<'a> Smtp<'a> {
    #[instrument(skip(self))]
    fn send_email(&self, to: &Email, subject: &str, body: String) -> Result<()> {
        let formated_subject = if !self.issuer.is_empty() {
            format!("[{}] {}", self.issuer, subject)
        } else {
            subject.to_string()
        };

        let to = to.as_ref().parse().map_err(|err: AddressError| {
            error!(
                to = to.as_ref(),
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
    fn send_credentials_verification_email(&self, email: &Email, token: &str) -> Result<()> {
        let mut context = Context::new();
        context.insert("name", email.as_ref().split('@').collect::<Vec<&str>>()[0]);
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
    fn send_credentials_reset_email(&self, email: &Email, token: &str) -> Result<()> {
        let mut context = Context::new();
        context.insert("name", email.as_ref().split('@').collect::<Vec<&str>>()[0]);
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
