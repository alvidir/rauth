use std::collections::HashMap;
use std::collections::hash_map;
use std::error::Error;
use super::{app, secret};
use crate::token::Token;

const ERR_NO_NAMESPACE: &str = "Namespace not found";
const ERR_NAMESPACE_ALREADY_EXISTS: &str = "The provided application already has an namespace";
const ERR_NAMESPACE_BUILD: &str = "Something has failed while building namespace";
const ERR_TOKEN_ALREADY_EXISTS: &str = "The namespace already has a dir for the provided token";
const ERR_USER_HAS_DIR: &str = "User already has a directory in this namespace";

static mut INSTANCE: Option<Box<dyn Factory>> = None;

pub trait Ctrl {
    fn get_id(&self) -> i32;
    fn get_label(&self) -> &str;
    fn set_token(&mut self,  cookie: Token, dir: Token,) -> Result<(), Box<dyn Error>>;
    fn get_secret(&self) -> &Box<dyn secret::Ctrl>;
    fn delete_token(&mut self, cookie: &Token) -> Option<Token>;
    fn get_token(&self, cookie: &Token) -> Option<&Token>;
    fn get_dirs_iter(&self) -> hash_map::Iter<Token, Token>;
}

pub trait Factory {
    fn new_namespace(&mut self, client: Box<dyn app::Ctrl>, secret: Box<dyn secret::Ctrl>) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>>;
    fn get_by_label(&mut self, label: &str) -> Option<&mut Box<dyn Ctrl>>;
    fn get_by_id(&mut self, app: i32) ->  Option<&mut Box<dyn Ctrl>>;
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
    fn new_namespace(&mut self, app: Box<dyn app::Ctrl>, secret: Box<dyn secret::Ctrl>) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>> {
        let label = app.get_label().to_string();

        if let None = self.allnp.get(&label) {
            //let client_id = app.get_client_id();
            //let secret = secret::find_by_client_and_name(client_id, default::RSA_NAME)?;
            let np = Namespace::new(app, secret);

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

    fn get_by_id(&mut self, target: i32) ->  Option<&mut Box<dyn Ctrl>> {
        if let Some((_, np)) = self.allnp.iter_mut().find(|(_, np)| np.get_id() == target) {
            Some(np)
        } else {
            None
        }
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
    dirs: HashMap<Token, Token>,
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
    fn get_id(&self) -> i32 {
        self.app.get_id()
    }

    fn get_label(&self) -> &str {
        self.app.get_label()
    }

    fn get_secret(&self) -> &Box<dyn secret::Ctrl> {
        &self.public
    }

    fn set_token(&mut self,  cookie: Token, dir: Token) -> Result<(), Box<dyn Error>> {
        if let Some(_) = self.dirs.get(&cookie) {
            return Err(ERR_TOKEN_ALREADY_EXISTS.into());
        }

        if let Some(_) = self.dirs.iter().find(|(_, d)| d.as_str() == dir.to_string()) {
            return Err(ERR_USER_HAS_DIR.into());
        }

        self.dirs.insert(cookie, dir);
        Ok(())
    }

    fn delete_token(&mut self, cookie: &Token) -> Option<Token> {
        self.dirs.remove(cookie)
    }

    fn get_token(&self, cookie: &Token) -> Option<&Token> {
        self.dirs.get(cookie)
    }

    fn get_dirs_iter(&self) -> hash_map::Iter<Token, Token> {
        self.dirs.iter()
    }
}