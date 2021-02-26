use crate::token::Token;
use std::error::Error;
use crate::models::{user, app, session, secret, namesp, Gateway};
use crate::models::app::Ctrl as AppCtrl;
use crate::models::secret::Ctrl as SecretCtrl;
use crate::models::namesp::Ctrl as NpCtrl;
use crate::regex::*;
use crate::default;

const ERR_IDENT_NOT_MATCH: &str = "The provided indentity is not of the expected type";
const ERR_PWD_NOT_MATCH: &str = "The provided password does not match";
const ERR_SIGNATURE_HAS_FAILED: &str = "Signature verifier has failed";

pub struct TxDelete<'a> {
    label: &'a str,
    dust: &'a [u8],
    firm: &'a [u8],
}

impl<'a> TxDelete<'a> {
    pub fn new(label: &'a str, dust: &'a [u8], firm: &'a [u8]) -> Self {
        TxDelete{
            label: label,
            dust: dust,
            firm: firm,
        }
    }

    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        println!("Got a Delete request from app {} ", self.label);

        let app = app::find_by_label(self.label)?;
        let secret = secret::find_by_client_and_name(app.get_client_id(), default::RSA_NAME)?;
        let mut verifier = secret.get_verifier()?;
        verifier.update(self.label.as_bytes())?;
        verifier.update(self.dust)?;

        if !verifier.verify(self.firm)? {
            return Err(ERR_SIGNATURE_HAS_FAILED.into());
        }

        if let Some(np) = namesp::get_instance().get_by_label(self.label) {
            // application is using a namespace
            for (cookie, token) in np.get_dirs_iter()  {
                // each opened directory must be deleted
                if let Some(sess) = session::get_instance().get_by_cookie(cookie) {
                    sess.delete_directory(token);
                }
            }

            namesp::get_instance().destroy_namespace(self.label)?;
        }

        /** MondoDB documents related to this application must be deleted as well */

        secret.delete()?;
        app.delete()?;
        Ok(())
    }
}