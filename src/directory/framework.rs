use std::error::Error;
use std::time::SystemTime;
use serde::{Serialize, Deserialize};
use bson::oid::ObjectId;
use bson::Bson;

use crate::mongo;
use crate::metadata::domain::Metadata;
use crate::constants::ERR_NOT_FOUND;
use super::domain::{Directory, DirectoryRepository};

const COLLECTION_NAME: &str = "directories";

#[derive(Serialize, Deserialize, Debug)]
struct MongoDirectoryMetadata {
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

#[derive(Serialize, Deserialize, Debug)]
struct MongoDirectory {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user: i32,
    pub app: i32,
    pub meta: MongoDirectoryMetadata,
}

pub struct MongoDirectoryRepository {}

impl MongoDirectoryRepository {
    fn builder(loaded_dir: bson::Document) -> Result<Directory, Box<dyn Error>> {
        let mongo_dir: MongoDirectory = bson::from_bson(Bson::Document(loaded_dir))?;
        
        let id: String;
        if let Some(dir_id) = mongo_dir.id {
            id = dir_id.to_hex();
        } else {
            return Err(ERR_NOT_FOUND.into());
        }

        let dir = Directory {
            id: id,
            user: mongo_dir.user,
            app: mongo_dir.app,
            deadline: SystemTime::UNIX_EPOCH,
            meta: Metadata {
                id: 0,
                created_at: mongo_dir.meta.created_at,
                updated_at: mongo_dir.meta.updated_at,
            },
        };

        return Ok(dir);
    }
}

impl DirectoryRepository for &MongoDirectoryRepository {
    fn find(&self, target: &str) -> Result<Directory, Box<dyn Error>>  {
        let loaded_dir_opt = mongo::open_stream(COLLECTION_NAME)
            .find_one(Some(doc! { "_id":  target }), None)?;

        if let Some(loaded_dir) = loaded_dir_opt {
            return MongoDirectoryRepository::builder(loaded_dir);
        }

        Err(ERR_NOT_FOUND.into())
    }

    fn find_by_user_and_app(&self, user_id: i32, app_id: i32) -> Result<Directory, Box<dyn Error>> {
        let loaded_dir_opt = mongo::open_stream(COLLECTION_NAME)
            .find_one(Some(doc! { "user":  user_id, "app": app_id }), None)?;

        if let Some(loaded_dir) = loaded_dir_opt {
            return MongoDirectoryRepository::builder(loaded_dir);
        }

        Err(ERR_NOT_FOUND.into())
    }

    fn save(&self, dir: &mut Directory) -> Result<(), Box<dyn Error>> {
        let mongo_meta = MongoDirectoryMetadata {
            created_at: dir.meta.created_at,
            updated_at: dir.meta.updated_at,
        };

        let mut id_opt = None;
        if dir.id.len() > 0 {
            let bson_id = ObjectId::with_string(&dir.id)?;
            id_opt = Some(bson_id);
        }

        let mongo_dir = MongoDirectory {
            id: id_opt,
            user: dir.user,
            app: dir.app,
            meta: mongo_meta,
        };

        let serialized = bson::to_bson(&mongo_dir)?;
        let document: &bson::Document;
        if let Some(doc) = serialized.as_document() {
            document = doc;
        } else {
            return Err("could not parse document".into());
        }

        if let Some(target) = mongo_dir.id {         
            mongo::open_stream(COLLECTION_NAME)
                .update_one(doc!{"_id": target.clone()}, document.to_owned(), None)?;
            
        } else {
            let result = mongo::open_stream(COLLECTION_NAME)
                .insert_one(document.to_owned(), None)?;

            let dir_id_opt = result
                .inserted_id
                .as_object_id();

            if let Some(dir_id) = dir_id_opt {
                dir.id = dir_id.to_hex();
            } else {
                return Err("retrieved id is not of the type ObjectId".into());
            }
        }

        Ok(())
    }

    fn delete(&self, dir: &Directory) -> Result<(), Box<dyn Error>> {
        let bson_id = ObjectId::with_string(&dir.id)?;
        mongo::open_stream(COLLECTION_NAME)
            .delete_one(doc!{"_id": bson_id}, None)?;

        Ok(())
    }
}