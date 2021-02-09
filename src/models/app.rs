use diesel::NotFound;
use std::error::Error;
use crate::schema::apps;
use crate::models::client;
use crate::models::enums;
use crate::regex::*;
use crate::diesel::prelude::*;
use crate::postgres::*;
use crate::token;
extern crate diesel;

const LABEL_LENGTH: usize = 15;

pub trait Ctrl {
    fn get_url(&self) -> &str;
    fn get_name(&self) -> &str;
    fn get_label(&self) -> &str;
}

//pub fn find_by_id(target: i32) -> Result<Box<impl Ctrl + super::Gateway>, Box<dyn Error>>  {
//    use crate::schema::apps::dsl::*;
//
//    let connection = open_stream();
//    let results = apps.filter(id.eq(target))
//        .load::<App>(connection)?;
//
//    if results.len() > 0 {
//        let client = client::find_by_id(results[0].client_id)?;
//        let wrapper = results[0].build(client)?;
//        Ok(Box::new(wrapper))
//    } else {
//        Err(Box::new(NotFound))
//    }
//}

pub fn find_by_label<'a>(target: &'a str, privileged: bool) -> Result<Box<impl Ctrl + super::Gateway>, Box<dyn Error>>  {
    use crate::schema::apps::dsl::*;

    let connection = open_stream();
    let results = apps.filter(label.eq(target))
        .load::<App>(connection)?;

    if results.len() > 0 {
        let client = client::find_by_id(results[0].client_id, privileged)?;
        let wrapper = results[0].build(client)?;
        Ok(Box::new(wrapper))
    } else {
        Err(Box::new(NotFound))
    }
}

//pub fn find_by_name<'a>(target: &'a str) -> Result<Box<impl Ctrl + super::Gateway>, Box<dyn Error>>  {
//    use crate::schema::apps::dsl::*;
//
//    let client = client::find_by_name(target)?;
//    if client.get_kind() != enums::Kind::APP {
//        let msg = format!("Client {:?} is not of the type {:?}", client.get_name(), enums::Kind::APP);
//        return Err(msg.into());
//    }
//
//    let connection = open_stream();
//    let results = apps.filter(client_id.eq(client.get_id()))
//        .load::<App>(connection)?;
//
//    if results.len() > 0 {
//        let wrapper = results[0].build(client)?;
//        Ok(Box::new(wrapper))
//    } else {
//        Err(Box::new(NotFound))
//    }
//}


//pub fn find_by_url<'a>(target: &'a str) -> Result<Box<impl Ctrl + super::Gateway>, Box<dyn Error>>  {
//    use crate::schema::apps::dsl::*;
//
//    let connection = open_stream();
//    let results = apps.filter(url.eq(target))
//        .load::<App>(connection)?;
//
//    if results.len() > 0 {
//        let client = client::find_by_id(results[0].client_id)?;
//        let wrapper = results[0].build(client)?;
//        Ok(Box::new(wrapper))
//    } else {
//        Err(Box::new(NotFound))
//    }
//}

#[derive(Queryable, Insertable, Associations)]
#[belongs_to(Client<'_>)]
#[derive(Clone)]
#[table_name = "apps"]
pub struct App {
    pub id: i32,
    pub client_id: i32,
    pub label: String,
    pub url: String,
    pub description: Option<String>,
}

#[derive(Insertable)]
#[table_name="apps"]
struct NewApp<'a> {
    pub client_id: i32,
    pub label: &'a str,
    pub url: &'a str,
    pub description: Option<&'a str>,
}

impl App {
    pub fn new<'a>(name: &'a str, url: &'a str) -> Result<Box<impl Ctrl + super::Gateway>, Box<dyn Error>> {
        match_url(url)?;

        let kind_id = enums::Kind::APP.to_int32();
        let client: Box<dyn client::Ctrl> = client::Client::new(kind_id, name)?;
        let label = token::Token::new(LABEL_LENGTH).to_string();
        let new_app = NewApp {
            client_id: client.get_id(),
            label: &label,
            url: url,
            description: None,
        };

        let connection = open_stream();
        let result = diesel::insert_into(apps::table)
            .values(&new_app)
            .get_result::<App>(connection)?;

        let wrapper = result.build(client)?;
        Ok(Box::new(wrapper))
    }

    fn build(&self, client: Box<dyn client::Ctrl>) -> Result<Wrapper, Box<dyn Error>> {
        Ok(Wrapper{
            app: self.clone(),
            client: client,
        })
    }
}

pub struct Wrapper {
    app: App,
    client: Box<dyn client::Ctrl>,
}

impl Ctrl for Wrapper {
    fn get_url(&self) -> &str {
        &self.app.url
    }

    fn get_name(&self) -> &str {
        self.client.get_name()
    }

    fn get_label(&self) -> &str {
        &self.app.label
    }
}

impl super::Gateway for Wrapper {
    fn delete(&self) -> Result<(), Box<dyn Error>> {
        use crate::schema::apps::dsl::*;

        let connection = open_stream();
        diesel::delete(
            apps.filter(
                id.eq(self.app.id)
            )
        ).execute(connection)?;

        self.client.get_gateway().delete()
    }
}