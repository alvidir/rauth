use diesel::NotFound;
use std::error::Error;
use crate::schema::apps;
use crate::models::client;
use crate::models::enums;
use crate::regex::*;
use crate::diesel::prelude::*;
use crate::postgres::*;
use crate::token;
use super::client::Ctrl as ClientCtrl;
extern crate diesel;
use crate::default;

pub trait Ctrl {
    fn get_id(&self) -> i32;
    fn get_url(&self) -> &str;
    fn get_name(&self) -> &str;
    fn get_label(&self) -> &str;
    fn get_descr(&self) -> &str;
    fn get_client_id(&self) -> i32;
}

//pub fn find_by_id(target: i32) -> Result<Box<impl Ctrl + super::Gateway>, Box<dyn Error>>  {
//    use crate::schema::apps::dsl::*;
//
//    let connection = open_stream().get()?;
//    let results = apps.filter(id.eq(target))
//        .load::<App>(&connection)?;
//
//    if results.len() > 0 {
//        let client = client::find_by_id(results[0].client_id)?;
//        let wrapper = results[0].build(client)?;
//        Ok(Box::new(wrapper))
//    } else {
//        Err(Box::new(NotFound))
//    }
//}

pub fn find_by_label<'a>(target: &'a str) -> Result<Box<impl Ctrl + super::Gateway>, Box<dyn Error>>  {
    use crate::schema::apps::dsl::*;

    let results = { // block is required because of connection release
        let connection = open_stream().get()?;
        apps.filter(label.eq(target))
            .load::<App>(&connection)?
    };

    if results.len() > 0 {
        let client = client::find_by_id(results[0].client_id)?;
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
//    let connection = open_stream().get()?;
//    let results = apps.filter(client_id.eq(client.get_id()))
//        .load::<App>(&connection)?;
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
//    let connection = open_stream().get()?;
//    let results = apps.filter(url.eq(target))
//        .load::<App>(&connection)?;
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
#[derive(Identifiable)]
#[belongs_to(Client<'_>)]
#[derive(Clone)]
#[table_name = "apps"]
pub struct App {
    pub id: i32,
    pub client_id: i32,
    pub label: String,
    pub url: String,
    pub description: String,
}

#[derive(Insertable)]
#[table_name="apps"]
struct NewApp<'a> {
    pub client_id: i32,
    pub label: &'a str,
    pub url: &'a str,
    pub description: &'a str,
}

impl App {
    pub fn new<'a>(name: &'a str, url: &'a str, descr: &'a str) -> Result<Box<impl Ctrl + super::Gateway>, Box<dyn Error>> {
        match_url(url)?;

        let kind_id = enums::Kind::APP.to_int32();
        let client: client::Wrapper = client::Client::new(kind_id, name)?;
        
        let label = token::Token::new(default::TOKEN_LEN).to_string();
        let app = App {
            id: 0,
            client_id: 0,
            label: label,
            url: url.to_string(),
            description: descr.to_string(),
        };

        let wrapper = app.build(client)?;
        Ok(Box::new(wrapper))
    }

    fn build(&self, client: client::Wrapper) -> Result<Wrapper, Box<dyn Error>> {
        Ok(Wrapper{
            app: self.clone(),
            client: client,
        })
    }
}

pub struct Wrapper {
    app: App,
    client: client::Wrapper,
}

impl Ctrl for Wrapper {
    fn get_id(&self) -> i32 {
        self.app.id
    }

    fn get_url(&self) -> &str {
        &self.app.url
    }

    fn get_name(&self) -> &str {
        self.client.get_name()
    }

    fn get_label(&self) -> &str {
        &self.app.label
    }

    fn get_descr(&self) -> &str {
        &self.app.description
    }

    fn get_client_id(&self) -> i32 {
        self.client.get_id()
    }
}

impl super::Gateway for Wrapper {
    fn select(&mut self) -> Result<(), Box<dyn Error>> {
        Err("".into())
    }
    
    fn insert(&mut self) -> Result<(), Box<dyn Error>> {
        self.client.insert()?;

        let new_app = NewApp {
            client_id: self.client.get_id(),
            label: &self.app.label,
            url: &self.app.url,
            description: &self.app.description,
        };

        let result = { // block is required because of connection release
            let connection = open_stream().get()?;
            diesel::insert_into(apps::table)
            .values(&new_app)
            .get_result::<App>(&connection)?
        };

        self.app.id = result.id;
        self.app.client_id = result.client_id;
        Ok(())
    }
    
    fn update(&mut self) -> Result<(), Box<dyn Error>> {
        { // block is required because of connection release
            let connection = open_stream().get()?;
            diesel::update(&self.app)
            .set((apps::url.eq(&self.app.url),
                  apps::description.eq(&self.app.description)))
            .execute(&connection)?;
        }

        Ok(())
    }
    
    fn delete(&self) -> Result<(), Box<dyn Error>> {
        use crate::schema::apps::dsl::*;

        { // block is required because of connection release
            let connection = open_stream().get()?;
            diesel::delete(
                apps.filter(
                    id.eq(self.app.id)
                )
            ).execute(&connection)?;
        }

        self.client.delete()
    }
}