use mongodb::bson::Document;
use std::collections::HashMap;
use std::time::Duration;
use std::time::SystemTime;
use std::error::Error;
use super::{app, secret};
use super::secret::Ctrl as SecretCtrl;
use crate::token::Token;
use serde::{Deserialize, Serialize};
use crate::transactions::register::DEFAULT_PKEY_NAME;

const PIN_LEN: usize = 8;
const ERR_NO_NAMESPACE: &str = "Namespace not found";
const ERR_NAMESPACE_ALREADY_EXISTS: &str = "The provided application already has an namespace";
const ERR_NAMESPACE_BUILD: &str = "Something has failed while building namespace";

static mut INSTANCE: Option<Box<dyn Factory>> = None;

pub trait Ctrl {
}

pub trait Factory {
    fn new_directory(&mut self, client: Box<dyn app::Ctrl>) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>>;
    fn get_by_label(&mut self, label: &str) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>>;
    fn destroy_dir(&mut self, label: &str) -> Result<(), Box<dyn Error>>;
}

pub fn get_instance<'a>() -> &'a mut Box<dyn Factory> {
    let provider: &mut Option<Box<dyn Factory>>;
    unsafe {
        provider = &mut INSTANCE
    }

    match provider {
        Some(ctrl) => {
            ctrl
        },
        None => {
            let instance = Provider::new();
            
            unsafe {
                INSTANCE = Some(Box::new(instance));
            }
            
            get_instance()
        },
    }
}

struct Provider {
    // all app's namespaces sorted by app's label
    allnp: HashMap<String, Box<dyn Ctrl>>,
}

impl Provider {
    fn new() -> impl Factory {
        Provider{
            allnp: HashMap::new(),        
        }
    }
}

impl Factory for Provider {
    fn new_directory(&mut self, client: Box<dyn app::Ctrl>) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>> {
        let label = client.get_label().to_string();

        if let None = self.allnp.get(&label) {
            let client_id = client.get_client_id();
            let secret = secret::find_by_client_and_name(client_id, DEFAULT_PKEY_NAME)?;
            let np = Namespace::new(client, secret);

            self.allnp.insert(label.clone(), Box::new(np));
            if let Some(np) = self.allnp.get_mut(&label) {
                Ok(np)
            } else {
                let msg = format!("{}", ERR_NAMESPACE_BUILD);
                Err(msg.into())
            }

        } else {
            // checking if there is already a session for the provided email
            let msg = format!("{}", ERR_NAMESPACE_ALREADY_EXISTS);
            Err(msg.into())
        }
    }

    fn get_by_label(&mut self, label: &str) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>> {
        if let Some(np) = self.allnp.get_mut(label) {
            Ok(np)
        } else {
            let msg = format!("{}", ERR_NO_NAMESPACE);
            Err(msg.into())
        }
    }

    fn destroy_dir(&mut self, label: &str) -> Result<(), Box<dyn Error>> {
        if let Some(_) = self.allnp.remove(label) {
            Ok(())
        } else {
            let msg = format!("{}", ERR_NO_NAMESPACE);
            Err(msg.into())
        }
    }
}

#[derive(Serialize, Deserialize)]
struct NewDir {
    user_id: i32,
    app_id: i32,
    data: Document,
}

struct Namespace<'a> {
    app: Box<dyn app::Ctrl>,
    public: Box<dyn secret::Ctrl>,
    dirs: HashMap<Token, &'a Document>,
    pin: Token, // random string (must change for each request)
}

impl<'a> Namespace<'a> {
    pub fn new(app: Box<dyn app::Ctrl>, secret: Box<dyn secret::Ctrl>) -> Self {
        Namespace{
            app: app,
            public: secret,
            pin: Token::new(PIN_LEN),
            dirs: HashMap::new(),
        }
    }
}

impl<'a> Ctrl for Namespace<'a> {
    
}

impl <'a> super::Gateway for Document {
    fn insert(&mut self) -> Result<(), Box<dyn Error>> {
        Err("".into())
    }

    fn update(&mut self) -> Result<(), Box<dyn Error>> {
        Err("".into())
    }

    fn delete(&self) -> Result<(), Box<dyn Error>> {
        Err("".into())
    }
}