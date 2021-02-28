use std::error::Error;
use crate::models::{app, secret};
use crate::models::app::Ctrl as AppCtrl;
use crate::models::secret::Ctrl as SecretCtrl;
use crate::models::Gateway;
use crate::proto::app_proto::RegisterResponse;
use crate::default;

const ERR_SIGNATURE_HAS_FAILED: &str = "Signature verifier has failed";

pub struct TxRegister<'a> {
    name: &'a str,
    url: &'a str,
    descr: &'a str,
    public: &'a [u8],
    firm: &'a [u8],
}

impl<'a> TxRegister<'a> {
    pub fn new(name: &'a str, url: &'a str, descr: &'a str, public:&'a [u8], firm: &'a [u8]) -> Self {
        TxRegister{
            name: name,
            url: url,
            descr: descr,
            public: public,
            firm: firm,
        }
    }

    pub fn execute(&self) -> Result<RegisterResponse, Box<dyn Error>> {
        println!("Got a Register request for application {} ", self.name);

        let mut app = app::App::new(self.name, self.url, self.descr)?;
        let aux_secret = secret::Secret::new(0, default::RSA_NAME, self.public)?;
        let mut verifier = aux_secret.get_verifier()?;
        verifier.update(self.name.as_bytes())?;
        verifier.update(self.url.as_bytes())?;
        verifier.update(self.descr.as_bytes())?;
        verifier.update(self.public)?;

        if !verifier.verify(self.firm)? {
            return Err(ERR_SIGNATURE_HAS_FAILED.into());
        }
        
        app.insert()?;
        
        let mut secret = secret::Secret::new(app.get_client_id(), default::RSA_NAME, self.public)?;
        if let Err(err) = secret.insert() {
            if let Err(nerr) = app.delete() {
                return Err(nerr);
            }

            return Err(err);
        }

        let encrypted = secret.encrypt(app.get_label().as_bytes())?;
        Ok(RegisterResponse{
            label: encrypted,
        })
    }
}