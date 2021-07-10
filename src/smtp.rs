use std::error::Error;
use std::env;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::constants;

const ERR_NOT_USERNAME: &str = "SMTP username must be set";
const ERR_NOT_PASSWORD: &str = "SMTP password must be set";
const ERR_NOT_URL: &str = "SMTP transport url must be set";
const ERR_NOT_ORIGIN: &str = "SMTP origin email must be set";

struct SMTP {
    pub mailer: SmtpTransport,
    pub from: String,
}

lazy_static! {
    static ref SMTP_INSTANCE: SMTP = {
        let smtp_username = env::var(constants::ENV_SMTP_USERNAME).expect(ERR_NOT_USERNAME);
        let smtp_password = env::var(constants::ENV_SMTP_PASSWORD).expect(ERR_NOT_PASSWORD);
        let smtp_transport = env::var(constants::ENV_SMTP_TRANSPORT).expect(ERR_NOT_URL);
        let smtp_from = env::var(constants::ENV_SMTP_ORIGIN).expect(ERR_NOT_ORIGIN);

        let creds = Credentials::new(smtp_username, smtp_password);
        let mailer = SmtpTransport::relay(&smtp_transport)
            .unwrap()
            .credentials(creds)
            .build();

        SMTP {
            mailer: mailer,
            from: smtp_from,
        }
    };    
}


pub fn send_email(to: &str, subject: &str, body: &str) -> Result<(), Box<dyn Error>> {
    let email = Message::builder()
        .from(SMTP_INSTANCE.from.parse()?)
        .to(to.parse()?)
        .subject(subject)
        .body(body.to_string())?;

    SMTP_INSTANCE.mailer.send(&email)?;
    Ok(())
}