use std::error::Error;
use crate::models::{app, session, secret, namesp, Gateway};
use crate::models::app::Ctrl as AppCtrl;
use crate::models::secret::Ctrl as SecretCtrl;
use crate::default;
use crate::mongo;

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
        println!("Got an Account deletion request from app {} ", self.label);

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

        let coll_name = mongo::get_collection_name()?;
        let delete_result = mongo::open_stream(&coll_name).delete_many(
            doc! {
               "app_label": app.get_label(),
            },
            None,
        )?;

        println!("Deleted {} documents", delete_result.deleted_count);
        secret.delete()?;
        app.delete()?;
        Ok(())
    }
}