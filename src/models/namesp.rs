use std::collections::HashMap;
use std::error::Error;
use super::{app, secret, dir, session};
use super::secret::Ctrl as SecretCtrl;
use super::dir::Ctrl as DirCtrl;
use crate::token::Token;
use std::collections::hash_map::Iter;
use crate::default;

const PIN_LEN: usize = 8;
const ERR_NO_NAMESPACE: &str = "Namespace not found";
const ERR_NAMESPACE_ALREADY_EXISTS: &str = "The provided application already has an namespace";
const ERR_NAMESPACE_BUILD: &str = "Something has failed while building namespace";
const ERR_TOKEN_ALREADY_EXISTS: &str = "The namespace already has a dir for the provided token";
const ERR_USER_HAS_DIR: &str = "User already has a directory in this namespace";
const ERR_DIR_NOT_EXISTS: &str = "There is no directory for the provuided token";

static mut INSTANCE: Option<Box<dyn Factory>> = None;

pub trait Ctrl {
    fn new_directory(&mut self, sess: &mut Box<dyn session::Ctrl>) -> Result<Token, Box<dyn Error>>;
    fn get_directory(&self, token: Token) -> Result<Box<&dyn dir::Ctrl>, Box<dyn Error>>;
    fn get_all_tokens(&self) -> Vec<Token>;
    fn delete_directory(&mut self, token: &Token) -> Result<Box<dyn super::Gateway>, Box<dyn Error>>;
    fn get_secret(&self) -> &Box<dyn secret::Ctrl>;
}

pub trait Factory {
    fn new_namespace(&mut self, client: Box<dyn app::Ctrl>) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>>;
    fn get_by_label(&mut self, label: &str) -> Option<&mut Box<dyn Ctrl>>;
    fn destroy_namespace(&mut self, label: &str) -> Result<(), Box<dyn Error>>;
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
    fn new_namespace(&mut self, appl: Box<dyn app::Ctrl>) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>> {
        let label = appl.get_label().to_string();

        if let None = self.allnp.get(&label) {
            let client_id = appl.get_client_id();
            let secret = secret::find_by_client_and_name(client_id, default::RSA_NAME)?;
            let np = Namespace::new(appl, secret);

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

    fn get_by_label(&mut self, label: &str) ->  Option<&mut Box<dyn Ctrl>> {
        self.allnp.get_mut(label)
    }

    fn destroy_namespace(&mut self, label: &str) -> Result<(), Box<dyn Error>> {
        if let Some(_) = self.allnp.remove(label) {
            Ok(())
        } else {
            let msg = format!("{}", ERR_NO_NAMESPACE);
            Err(msg.into())
        }
    }
}

struct Namespace {
    app: Box<dyn app::Ctrl>,
    public: Box<dyn secret::Ctrl>,
    dirs: HashMap<Token, dir::Dir>,
}

impl Namespace {
    pub fn new(app: Box<dyn app::Ctrl>, secret: Box<dyn secret::Ctrl>) -> Self {
        Namespace{
            app: app,
            public: secret,
            dirs: HashMap::new(),
        }
    }
}

impl Ctrl for Namespace {
    fn new_directory(&mut self, sess: &mut Box<dyn session::Ctrl>) -> Result<Token, Box<dyn Error>> {
        let token = sess.attach_label(self.app.get_label())?;
        if let Some(_) = self.dirs.get(&token) {
            return Err(ERR_TOKEN_ALREADY_EXISTS.into());
        }

        if let Some(_) = self.dirs.iter().find(|(_, d)| d.get_user_id() == sess.get_user_id()) {
            return Err(ERR_USER_HAS_DIR.into());
        }

        let dir = dir::Dir::new(sess.get_user_id(), self.app.get_id());
        self.dirs.insert(token.clone(), dir);
        Ok(token)
    }

    fn get_directory(&self, token: Token) -> Result<Box<&dyn dir::Ctrl>, Box<dyn Error>> {
        if let Some(dir) = self.dirs.get(&token) {
            Ok(Box::new(dir))
        } else {
            Err(ERR_DIR_NOT_EXISTS.into())
        }
    }

    fn get_all_tokens(&self) -> Vec<Token> {
        self.dirs.keys().cloned().collect()
    }

    fn get_secret(&self) -> &Box<dyn secret::Ctrl> {
        &self.public
    }

    fn delete_directory(&mut self, token: &Token) -> Result<Box<dyn super::Gateway>, Box<dyn Error>> {
        if let Some(dir) = self.dirs.remove(&token) {
            Ok(Box::new(dir))
        } else {
            Err(ERR_DIR_NOT_EXISTS.into())
        }
    }
}