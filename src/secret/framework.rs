use std::error::Error;
use std::time::SystemTime;
use serde::{Serialize, Deserialize};
use bson::oid::ObjectId;
use bson::Bson;

use crate::mongo;
use crate::metadata::domain::InnerMetadata;
use crate::constants::errors;
use super::domain::{Secret, SecretRepository};

const COLLECTION_NAME: &str = "secrets";

#[derive(Serialize, Deserialize, Debug)]
struct MongoSecretMetadata {
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

#[derive(Serialize, Deserialize, Debug)]
struct MongoSecret {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub data: Vec<u8>,
    pub meta: MongoSecretMetadata,
}

pub struct MongoSecretRepository;

impl SecretRepository for MongoSecretRepository {
    fn find(&self, target: &str) -> Result<Secret, Box<dyn Error>>  {
        let loaded_secret_opt = mongo::get_connection(COLLECTION_NAME)
            .find_one(Some(doc! { "_id":  target }), None)?;

        if let Some(loaded_secret) = loaded_secret_opt {
            let mongo_secret: MongoSecret = bson::from_bson(Bson::Document(loaded_secret))?;
            
            let id: String;
            if let Some(secret_id) = mongo_secret.id {
                id = secret_id.to_hex();
            } else {
                return Err(errors::NOT_FOUND.into());
            }

            let secret = Secret {
                id: id,
                data: mongo_secret.data,
                meta: InnerMetadata {
                    created_at: mongo_secret.meta.created_at,
                    updated_at: mongo_secret.meta.updated_at,
                },
            };

            return Ok(secret);
        }

        
        Err(errors::NOT_FOUND.into())
    }

    fn save(&self, secret: &mut Secret) -> Result<(), Box<dyn Error>> {
        let mongo_meta = MongoSecretMetadata {
            created_at: secret.meta.created_at,
            updated_at: secret.meta.updated_at,
        };

        let mut id_opt = None;
        if secret.id.len() > 0 {
            let bson_id = ObjectId::with_string(&secret.id)?;
            id_opt = Some(bson_id);
        }

        let mongo_secret = MongoSecret {
            id: id_opt,
            data: secret.data.to_vec(),
            meta: mongo_meta,
        };

        let serialized = bson::to_bson(&mongo_secret)?;
        let document: &bson::Document;
        if let Some(doc) = serialized.as_document() {
            document = doc;
        } else {
            return Err("could not parse document".into());
        }

        if let Some(target) = mongo_secret.id {         
            mongo::get_connection(COLLECTION_NAME)
                .update_one(doc!{"_id": target.clone()}, document.to_owned(), None)?;
            
        } else {
            let result = mongo::get_connection(COLLECTION_NAME)
                .insert_one(document.to_owned(), None)?;

            let secret_id_opt = result
                .inserted_id
                .as_object_id();

            if let Some(secret_id) = secret_id_opt {
                secret.id = secret_id.to_hex();
            } else {
                return Err("retrieved id is not of the type ObjectId".into());
            }
        }

        Ok(())
    }

    fn delete(&self, secret: &Secret) -> Result<(), Box<dyn Error>> {
        let bson_id = ObjectId::with_string(&secret.id)?;
        mongo::get_connection(COLLECTION_NAME)
            .delete_one(doc!{"_id": bson_id}, None)?;

        Ok(())
    }
}