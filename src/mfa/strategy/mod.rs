mod tp_app;
pub use tp_app::*;

mod email;
pub use email::*;

use super::{domain::Otp, error::Result};
use crate::user::domain::Email;

pub trait MailService {
    fn send_otp_email(&self, to: &Email, token: &Otp) -> Result<()>;
}
