use std::error::Error;
use std::time::{Duration, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use bson::oid::ObjectId;
use bson::{Bson, Document};

use crate::mongo;
use crate::metadata::domain::InnerMetadata;
use crate::constants::errors;
use super::domain::{Secret, SecretRepository};

const COLLECTION_NAME: &str = "secrets";

#[derive(Serialize, Deserialize, Debug)]
struct MongoSecretMetadata {
    pub created_at: f64,
    pub touch_at: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct MongoSecret {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub data: String,
    pub meta: MongoSecretMetadata,
}

pub struct MongoSecretRepository;

impl MongoSecretRepository {
    fn parse_secret(secret: &Secret) -> Result<Document, Box<dyn Error>> {
        let mongo_meta = MongoSecretMetadata {
            created_at: secret.meta.created_at.duration_since(UNIX_EPOCH)?.as_secs_f64(),
            touch_at: secret.meta.touch_at.duration_since(UNIX_EPOCH)?.as_secs_f64(),
        };

        let mut id_opt = None;
        if secret.id.len() > 0 {
            let bson_id = ObjectId::with_string(&secret.id)?;
            id_opt = Some(bson_id);
        }

        // BSON does not support unsigned type
        let data = match String::from_utf8(secret.data.clone()) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };

        let mongo_secret = MongoSecret {
            id: id_opt,
            data: data,
            meta: mongo_meta,
        };

        let serialized = bson::to_bson(&mongo_secret)?;
        if let Some(doc) = serialized.as_document() {
            return Ok(doc.clone());
        }
        
        Err(errors::PARSE_FAILED.into())
    }
}

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
                data: mongo_secret.data.as_bytes().to_vec(),
                meta: InnerMetadata {
                    created_at: UNIX_EPOCH + Duration::from_secs_f64(mongo_secret.meta.created_at),
                    touch_at: UNIX_EPOCH + Duration::from_secs_f64(mongo_secret.meta.touch_at),
                },
            };

            return Ok(secret);
        }

        
        Err(errors::NOT_FOUND.into())
    }

    fn create(&self, secret: &mut Secret) -> Result<(), Box<dyn Error>> {
        let document = MongoSecretRepository::parse_secret(secret)?;
        let result = mongo::get_connection(COLLECTION_NAME)
            .insert_one(document.to_owned(), None)?;

        let secret_id_opt = result
            .inserted_id
            .as_object_id();

        if let Some(secret_id) = secret_id_opt {
            secret.id = secret_id.to_hex();
            Ok(())
        } else {
            Err(errors::PARSE_FAILED.into())
        }
    }

    fn save(&self, secret: &Secret) -> Result<(), Box<dyn Error>> {
        let document = MongoSecretRepository::parse_secret(secret)?;       
        mongo::get_connection(COLLECTION_NAME)
            .update_one(doc!{"_id": secret.get_id()}, document.to_owned(), None)?;

        Ok(())
    }

    fn delete(&self, secret: &Secret) -> Result<(), Box<dyn Error>> {
        let bson_id = ObjectId::with_string(&secret.id)?;
        mongo::get_connection(COLLECTION_NAME)
            .delete_one(doc!{"_id": bson_id}, None)?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "integration-tests")]
mod tests {
    use super::super::domain::{Secret, SecretRepository};

    #[test]
    fn secret_create_ok() {
        let repo = super::MongoSecretRepository;
        let mut secret = Secret::new("secret".as_bytes());
        
        repo.create(&mut secret).unwrap();
        repo.delete(&secret).unwrap();
    }
}