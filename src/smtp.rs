use std::error::Error;
use std::env;
use lettre::smtp::authentication::Credentials;
use lettre::{SmtpClient, SmtpTransport, Transport};
use lettre_email::EmailBuilder;
use tera::{Tera, Context};

use crate::constants::environment;

lazy_static! {
    static ref TERA: Tera = {
        let templates = env::var(environment::TEMPLATES).unwrap();
        Tera::new(&templates).unwrap()
    };
}

fn get_mailer() -> Result<SmtpTransport, Box<dyn Error>> {
    let smtp_username = env::var(environment::SMTP_USERNAME)?;
    let smtp_password = env::var(environment::SMTP_PASSWORD)?;
    let smtp_transport = env::var(environment::SMTP_TRANSPORT)?;

    let creds = Credentials::new(smtp_username, smtp_password);
    let mailer = SmtpClient::new_simple(&smtp_transport)?
        .credentials(creds)
        .transport();

    Ok(mailer)
}

pub fn send_verification_email(to: &str, token: &str) -> Result<(), Box<dyn Error>> {
    let mut verify_url = env::var(environment::VERIFY_URL)?;
    verify_url = format!("{}?t={}", verify_url, token);

    let mut context = Context::new();
    context.insert("verify_url", &verify_url);
    
    const SUBJECT: &str = "Alvidir | Verification email";
    let body = TERA.render("verification_email.html", &context)?;
    if let Err(err) = send_email(to, SUBJECT, &body) {
        info!("got error {} while sending verification email to {}", err, to);
        return Err(err);
    }

    Ok(())
}


pub fn send_email(to: &str, subject: &str, body: &str) -> Result<(), Box<dyn Error>> {
    let from = env::var(environment::SMTP_ORIGIN)?;
    let email = EmailBuilder::new()
        .to(to)
        .from(from)
        .subject(subject)
        .html(body)
        .build()?;

    get_mailer()?.send(email.into())?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use std::env;
    use crate::constants::environment;
    use super::send_verification_email;

    #[test]
    fn send_verification_email_ok() {
        // seting up environment variables
        if let Err(_) = dotenv::dotenv() {
            warn!("no dotenv file has been found");
        }

        const TOKEN: &str = "dummytoken";
        let mailto = env::var(environment::SMTP_USERNAME).unwrap();
        send_verification_email(&mailto, TOKEN).unwrap();
    }
}