use crate::models::client::Controller as ClientController;
use crate::schema::apps;

pub trait Controller {
    fn get_description(&self) -> &str;
    fn get_addr(&self) -> &str;
}

#[derive(Queryable, Insertable, Associations)]
#[belongs_to(Client<'_>)]
#[derive(Clone)]
#[table_name = "apps"]
pub struct App {
    pub id: i32,
    pub description: String,
    pub url: String,
}

impl App {
    pub fn new(client: Box<dyn ClientController>, url: String) -> impl Controller {
        App{
            id: 0,
            description: "".to_string(),
            url: url,
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