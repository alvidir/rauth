use std::error::Error;
use tonic::{Request, Response, Status};
use diesel::NotFound;

use crate::diesel::prelude::*;
use crate::schema::apps::dsl::*;
use crate::postgres::*;
use crate::schema::apps;
use crate::secret::framework::MongoSecretRepository;
use crate::secret::domain::SecretRepository;
use crate::metadata::framework::PostgresMetadataRepository;
use crate::metadata::domain::MetadataRepository;
use crate::security;

use super::domain::{App, AppRepository};

// Import the generated rust code into module
mod proto {
    tonic::include_proto!("app");
}

// Proto generated server traits
use proto::app_service_server::AppService;
pub use proto::app_service_server::AppServiceServer;

// Proto message structs
use proto::{RegisterRequest, DeleteRequest};

pub struct AppServiceImplementation {
    app_repo: &'static PostgresAppRepository,
    secret_repo: &'static MongoSecretRepository,
    meta_repo: &'static PostgresMetadataRepository,
}

impl AppServiceImplementation {
    pub fn new(app_repo: &'static PostgresAppRepository,
               secret_repo: &'static MongoSecretRepository,
               meta_repo: &'static PostgresMetadataRepository) -> Self {
        AppServiceImplementation {
            app_repo: app_repo,
            secret_repo: secret_repo,
            meta_repo: meta_repo
        }
    }
}

#[tonic::async_trait]
impl AppService for AppServiceImplementation {
    async fn register(&self, request: Request<RegisterRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();

        let mut data: Vec<&[u8]> = Vec::new();
        data.push(&msg_ref.url.as_bytes());
        data.push(&msg_ref.public);
    
        // the application can only be registered if, and only if, the provided secret matches
        // the message signature; otherwise there is no way to ensure the secret is the app's one
        if let Err(err) = security::verify_ec_signature(&msg_ref.public, &msg_ref.firm, &data) {
            return Err(Status::unauthenticated(err.to_string()))
        }

        match super::application::app_register(&self.app_repo,
                                               &self.secret_repo,
                                               &self.meta_repo,
                                               &msg_ref.public,
                                               &msg_ref.url) {

            Err(err) => Err(Status::aborted(err.to_string())),
            Ok(()) => Ok(Response::new(())),
        }
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();

        let app_search = self.app_repo.find(&msg_ref.url);
        if let Err(err) = app_search {
            return Err(Status::not_found(err.to_string()));
        } 

        let app = app_search.unwrap();
        let pem = app.secret.get_data();
        
        let mut data: Vec<&[u8]> = Vec::new();
        data.push(&msg_ref.url.as_bytes());

        if let Err(err) = security::verify_ec_signature(pem, &msg_ref.firm, &data) {
            // the provided signature must be valid for the app's secret 
            return Err(Status::permission_denied(err.to_string()));
        }

        match super::application::app_delete(Box::new(self.app_repo),
                                             &msg_ref.url) {

            Err(err) => Err(Status::aborted(err.to_string())),
            Ok(()) => Ok(Response::new(())),
        }
    }
}

#[derive(Queryable, Insertable, Associations)]
#[derive(Identifiable)]
#[derive(AsChangeset)]
#[derive(Clone)]
#[table_name = "apps"]
struct PostgresApp {
    pub id: i32,
    pub url: String,
    pub secret_id: String,
    pub meta_id: i32,
}

#[derive(Insertable)]
#[derive(Clone)]
#[table_name = "apps"]
struct NewPostgresApp<'a> {
    pub url: &'a str,
    pub secret_id: &'a str,
    pub meta_id: i32,
}


pub struct PostgresAppRepository {
    secret_repo: &'static MongoSecretRepository,
    meta_repo: &'static PostgresMetadataRepository,
}

impl PostgresAppRepository {
    pub fn new(secret_repo: &'static MongoSecretRepository,
               meta_repo: &'static PostgresMetadataRepository) -> Self {
        PostgresAppRepository {
            secret_repo: secret_repo,
            meta_repo: meta_repo,
        }
    }
}

impl AppRepository for &PostgresAppRepository {
    fn find(&self, target: &str) -> Result<App, Box<dyn Error>>  {
        let results = { // block is required because of connection release
            let connection = open_stream().get()?;
            apps.filter(url.eq(target))
                 .load::<PostgresApp>(&connection)?
        };
    
        if results.len() == 0 {
            return Err(Box::new(NotFound));
        }

        let secret = self.secret_repo.find(&results[0].secret_id)?;
        let meta = self.meta_repo.find(results[0].meta_id)?;
        
        Ok(App{
            id: results[0].id,
            url: results[0].url.clone(),
            secret: secret,
            meta: meta,
        })
    }

    fn save(&self, app: &mut App) -> Result<(), Box<dyn Error>> {
        self.meta_repo.save(&mut app.meta)?;

        if app.id == 0 { // create user
            let new_app = NewPostgresApp {
                url: &app.url,
                secret_id: "",
                meta_id: app.meta.id,
            };
    
            let result = { // block is required because of connection release
                let connection = open_stream().get()?;
                diesel::insert_into(apps::table)
                    .values(&new_app)
                    .get_result::<PostgresApp>(&connection)?
            };
    
            app.id = result.id;
            Ok(())

        } else { // update user
            let pg_app = PostgresApp {
                id: app.id,
                url: app.url.clone(),
                secret_id: "".to_string(),
                meta_id: app.meta.id,
            };
            
            { // block is required because of connection release            
                let connection = open_stream().get()?;
                diesel::update(apps)
                    .set(&pg_app)
                    .execute(&connection)?;
            }
    
            Ok(())
        }
    }

    fn delete(&self, app: &App) -> Result<(), Box<dyn Error>> {
        { // block is required because of connection release
            let connection = open_stream().get()?;
            let _result = diesel::delete(
                apps.filter(
                    id.eq(app.id)
                )
            ).execute(&connection)?;
        }

        Ok(())
    }
}