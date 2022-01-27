use std::error::Error;
use std::collections::BTreeMap;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{SmtpTransport, Message, Transport};
use tera::{Tera, Context};

pub struct Smtp<'a> {
    pub issuer: &'a str,
    pub origin: &'a str,
    mailer: SmtpTransport,
    tera: Tera,
}

impl<'a> Smtp<'a> {
    pub fn new(templates_path: &str, smtp_transport: String, smtp_credentials: Option<(String, String)>) -> Result<Self, Box<dyn Error>> {
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
            .body(body)?;
    
        self.mailer.send(&email)?;
        Ok(())
    }

    pub fn send_email_template(&self, to: &str, subject: &str, template: &str, context: BTreeMap<String, String>) -> Result<(), Box<dyn Error>> {
        let mut tera_context = Context::new();
        context.iter().for_each(|(key, value)| {
            tera_context.insert(key.to_string(), value);
        });
        
        let body = self.tera.render(template, &tera_context)?;
        self.send_email(to, &subject, body)
    }
}
