use std::error::Error;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{SmtpTransport, Message, Transport};
use lettre::message::SinglePart;
use tera::{Tera, Context};
use crate::constants;

pub trait Mailer {
    fn send_verification_email(&self, to: &str, token: &str) ->  Result<(), Box<dyn Error>>;
}

pub struct Smtp<'a> {
    pub issuer: &'a str,
    pub origin: &'a str,
    mailer: SmtpTransport,
    tera: Tera,
}

impl<'a> Smtp<'a> {
    pub fn new(templates_path: &str, smtp_transport: &str, smtp_credentials: Option<(String, String)>) -> Result<Self, Box<dyn Error>> {
        let tera = Tera::new(templates_path)?;
        let mut mailer = SmtpTransport::relay(&smtp_transport)?;
        if let Some(credentials) = smtp_credentials {
            let creds = Credentials::new(credentials.0, credentials.1);
            mailer = mailer.credentials(creds);
        }

        Ok(Smtp {
            issuer: "",
            origin: "",
            mailer: mailer.build(),
            tera: tera,
        })
    }

    pub fn send_email(&self, to: &str, subject: &str, body: String) -> Result<(), Box<dyn Error>> {
        let formated_subject = if self.issuer.len() > 0 {
            format!("[{}] {}", self.issuer, subject)
        } else {
            subject.to_string()
        };

        let email = Message::builder()
            .from(self.origin.parse()?)
            .to(to.parse()?)
            .subject(formated_subject)
            .singlepart(SinglePart::html(body))?;
    
        self.mailer.send(&email)?;
        Ok(())
    }
}

impl<'a> Mailer for Smtp<'a> {
    fn send_verification_email(&self, email: &str, token: &str) ->  Result<(), Box<dyn Error>> {
        let mut context = Context::new();
        context.insert("name", email.split("@").collect::<Vec<&str>>()[0]);
        context.insert("token", &base64::encode(token));

        const SUBJECT: &str = constants::VERIFICATION_EMAIL_SUBJECT;
        let body = self.tera.render(constants::VERIFICATION_EMAIL_TEMPLATE, &context)?;
        self.send_email(email, &SUBJECT, body)
    }
}

#[cfg(test)]
pub mod tests {
    use std::error::Error;
    use super::Mailer;

    pub struct MailerMock {
        pub force_fail: bool,
    }

    impl MailerMock {
        pub fn new() -> Self {
            MailerMock{
                force_fail: false,
            }
        }
    }

    impl Mailer for MailerMock {
        fn send_verification_email(&self, _: &str, _: &str) -> Result<(), Box<dyn Error>> {
            if self.force_fail {
                return Err("fail forced".into());
            }

            Ok(())
        }
    }
}