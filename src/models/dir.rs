use std::error::Error;
use serde::{Deserialize, Serialize};
use mongodb::bson;

pub trait Ctrl {
    fn get_user_id(&self) -> i32;
    fn get_label(&self) -> &str;
    fn get_data(&self) -> &bson::Document;
}

#[derive(Serialize, Deserialize)]
pub struct Dir {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<bson::oid::ObjectId>,
    user_id: i32,
    app_label: String,
    data: bson::Document,
}

impl Dir {
    pub fn new(user: i32, label: &str) -> Self {
        Dir{
            id: None,
            user_id: user,
            app_label: label.to_string(),
            data: bson::Document::new(),
        }
    }
}

impl Ctrl for Dir {
    fn get_user_id(&self) -> i32{
        self.user_id
    }

    fn get_label(&self) -> &str{
        &self.app_label
    }

    fn get_data(&self) -> &bson::Document {
        &self.data
    }

}

impl super::Gateway for Dir {
    fn select(&mut self) -> Result<(), Box<dyn Error>> {
        Err("".into())
    }

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
