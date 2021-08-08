use std::error::Error;
use tonic::{Request, Response, Status};
use diesel::NotFound;

use crate::diesel::prelude::*;
use crate::schema::apps::dsl::*;
use crate::postgres::*;
use crate::schema::apps;

use crate::metadata::get_repository as get_meta_repository;
use crate::secret::get_repository as get_secret_repository;

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

pub struct AppServiceImplementation;

#[tonic::async_trait]
impl AppService for AppServiceImplementation {
    async fn register(&self, request: Request<RegisterRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();

        match super::application::app_register(&msg_ref.url,
                                               &msg_ref.public,
                                               &msg_ref.firm) {

            Err(err) => Err(Status::aborted(err.to_string())),
            Ok(()) => Ok(Response::new(())),
        }
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();
        
        match super::application::app_delete(&msg_ref.url,
                                             &msg_ref.firm) {

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

pub struct PostgresAppRepository;

impl AppRepository for PostgresAppRepository {
    fn find(&self, target: &str) -> Result<App, Box<dyn Error>>  {
        let results = { // block is required because of connection release
            let connection = get_connection().get()?;
            apps.filter(url.eq(target))
                 .load::<PostgresApp>(&connection)?
        };
    
        if results.len() == 0 {
            return Err(Box::new(NotFound));
        }

        let secret = get_secret_repository().find(&results[0].secret_id)?;
        let meta = get_meta_repository().find(results[0].meta_id)?;
        
        Ok(App{
            id: results[0].id,
            url: results[0].url.clone(),
            secret: secret,
            meta: meta,
        })
    }

    fn create(&self, app: &mut App) -> Result<(), Box<dyn Error>> {
        let new_app = NewPostgresApp {
            url: &app.url,
            secret_id: "",
            meta_id: app.meta.get_id(),
        };

        let result = { // block is required because of connection release
            let connection = get_connection().get()?;
            diesel::insert_into(apps::table)
                .values(&new_app)
                .get_result::<PostgresApp>(&connection)?
        };

        app.id = result.id;
        Ok(())
    }

    fn save(&self, app: &App) -> Result<(), Box<dyn Error>> {
        get_meta_repository().save(&app.meta)?;
        let pg_app = PostgresApp {
            id: app.id,
            url: app.url.to_string(),
            secret_id: "".to_string(),
            meta_id: app.meta.get_id(),
        };
        
        { // block is required because of connection release            
            let connection = get_connection().get()?;
            diesel::update(apps)
                .set(&pg_app)
                .execute(&connection)?;
        }

        Ok(())
    }

    fn delete(&self, app: &App) -> Result<(), Box<dyn Error>> {
        { // block is required because of connection release
            let connection = get_connection().get()?;
            let _result = diesel::delete(
                apps.filter(
                    id.eq(app.id)
                )
            ).execute(&connection)?;
        }
        
        Ok(())
    }
}