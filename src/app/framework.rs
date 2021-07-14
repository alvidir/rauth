use std::error::Error;
use tonic::{Request, Response, Status};
use diesel::NotFound;

use crate::diesel::prelude::*;
use crate::schema::apps::dsl::*;
use crate::postgres::*;
use crate::schema::apps;
//use crate::secret::SECRET_REPOSITORY;
use crate::secret::domain::SecretRepository;
//use crate::metadata::METADATA_REPOSITORY;
use crate::metadata::domain::MetadataRepository;

use super::domain::{App, AppRepository};

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


pub struct PostgresAppRepository {}

impl AppRepository for PostgresAppRepository {
    fn find(target: &str) -> Result<App, Box<dyn Error>>  {
        // let results = { // block is required because of connection release
        //     let connection = open_stream().get()?;
        //     apps.filter(url.eq(url))
        //          .load::<PostgresApp>(&connection)?
        // };
    
        // if results.len() == 0 {
        //     return Err(Box::new(NotFound));
        // }

        // let secret = SECRET_REPOSITORY.find(&results[0].secret_id)?;
        // let meta = METADATA_REPOSITORY.find(results[0].meta_id)?;
        
        // Ok(App{
        //     id: results[0].id,
        //     url: results[0].url.clone(),
        //     secret: secret,
        //     meta: meta,
        // })

        Err("unimplemented".into())
    }

    fn save(app: &mut App) -> Result<(), Box<dyn Error>> {
        // METADATA_REPOSITORY.save(&mut app.meta)?;

        // if app.id == 0 { // create user
        //     let new_app = NewPostgresApp {
        //         url: &app.url,
        //         secret_id: "",
        //         meta_id: app.meta.id,
        //     };
    
        //     let result = { // block is required because of connection release
        //         let connection = open_stream().get()?;
        //         diesel::insert_into(apps::table)
        //             .values(&new_app)
        //             .get_result::<PostgresApp>(&connection)?
        //     };
    
        //     app.id = result.id;
        //     Ok(())

        // } else { // update user
        //     let pg_app = PostgresApp {
        //         id: app.id,
        //         url: app.url.clone(),
        //         secret_id: "".to_string(),
        //         meta_id: app.meta.id,
        //     };
            
        //     { // block is required because of connection release            
        //         let connection = open_stream().get()?;
        //         diesel::update(apps)
        //             .set(&pg_app)
        //             .execute(&connection)?;
        //     }
    
        //     Ok(())
        // }

        Err("unimplemented".into())
    }

    fn delete(app: &App) -> Result<(), Box<dyn Error>> {
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