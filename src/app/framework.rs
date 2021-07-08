use std::error::Error;
use tonic::{Request, Response, Status};
use diesel::NotFound;

use crate::diesel::prelude::*;
use crate::schema::apps::dsl::*;
use crate::postgres::*;
use crate::schema::apps;
use crate::meta::framework::PostgresMetadataRepository;

use super::domain::App;

// Import the generated rust code into module
mod proto {
    tonic::include_proto!("app");
}

// Proto generated server traits
use proto::app_service_server::AppService;
pub use proto::app_service_server::AppServiceServer;

// Proto message structs
use proto::{RegisterRequest, RegisterResponse, DeleteRequest};

#[derive(Default)]
pub struct AppServiceImplementation {}

#[tonic::async_trait]
impl AppService for AppServiceImplementation {
    async fn register(&self, request: Request<RegisterRequest>) -> Result<Response<RegisterResponse>, Status> {
        let msg_ref = request.into_inner();
        Err(Status::unimplemented(""))
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();
        Err(Status::unimplemented(""))
    }
}

#[derive(Queryable, Insertable, Associations)]
#[derive(Identifiable)]
#[derive(Clone)]
#[table_name = "apps"]
pub struct PostgresApp {
    pub id: i32,
    pub label: String,
    pub url: String,
    pub secret_id: String,
    pub meta_id: i32,
}

#[derive(Insertable)]
#[derive(Clone)]
#[table_name = "apps"]
pub struct NewPostgresApp {
    pub label: String,
    pub url: String,
    pub secret_id: String,
    pub meta_id: i32,
}


pub struct PostgresAppRepository {}

impl PostgresAppRepository {
    pub fn find(target: &str) -> Result<App, Box<dyn Error>>  {
        let results = { // block is required because of connection release
            let connection = open_stream().get()?;
            apps.filter(label.eq(target))
                 .load::<PostgresApp>(&connection)?
        };
    
        if results.len() == 0 {
            return Err(Box::new(NotFound));
        }

        let meta = PostgresMetadataRepository::find(results[0].meta_id)?;
        
        App::new(
            results[0].id,
            &results[0].label,
            &results[0].url,
            meta,
        )
    }
}