use crate::model::client::Controller as ClientController;

pub trait Controller {
    fn get_description(&self) -> &str;
    fn get_addr(&self) -> &str;
}

pub struct App {
    pub id: i32,
    pub description: String,
    pub url: String,
    client: Box<dyn ClientController>,
}

impl App {
    pub fn new(client: Box<dyn ClientController>, url: String) -> Self {
        App{
            id: 0,
            description: "".to_string(),
            url: url,
            client: client,
        }
    }
}

impl Controller for App {
    fn get_description(&self) -> &str {
        &self.description
    }

    fn get_addr(&self) -> &str {
        &self.url
    }
}