use std::error::Error;
use tonic::{Request, Response, Status};
use diesel::NotFound;

use crate::diesel::prelude::*;
use crate::schema::apps::dsl::*;
use crate::postgres::*;
use crate::schema::apps;

use crate::metadata::framework::META_REPO;
use crate::metadata::domain::MetadataRepository;
use crate::session::framework::SESS_REPO;
use crate::directory::framework::DIR_REPO;
use crate::secret::framework::SECRET_REPO;
use crate::secret::domain::SecretRepository;

lazy_static! {
    pub static ref APP_REPO: PostgresAppRepository = PostgresAppRepository;
}

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

        match super::application::app_register(&*APP_REPO,
                                               &*SECRET_REPO,
                                               &*META_REPO,
                                               &msg_ref.url,
                                               &msg_ref.public,
                                               &msg_ref.firm) {

            Err(err) => Err(Status::aborted(err.to_string())),
            Ok(()) => Ok(Response::new(())),
        }
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();
        
        match super::application::app_delete(&*APP_REPO,
                                             &msg_ref.url,
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

        let secret = SECRET_REPO.find(&results[0].secret_id)?;
        let meta = META_REPO.find(results[0].meta_id)?;
        
        Ok(App{
            id: results[0].id,
            url: results[0].url.clone(),
            secret: secret,
            meta: meta,

            //repo: &*APP_REPO,
        })
    }

    fn save(&self, app: &mut App) -> Result<(), Box<dyn Error>> {
        META_REPO.save(&mut app.meta)?;

        if app.id == 0 { // create user
            let new_app = NewPostgresApp {
                url: &app.url,
                secret_id: "",
                meta_id: app.meta.id,
            };
    
            let result = { // block is required because of connection release
                let connection = get_connection().get()?;
                diesel::insert_into(apps::table)
                    .values(&new_app)
                    .get_result::<PostgresApp>(&connection)?
            };
    
            app.id = result.id;
            Ok(())

        } else { // update user
            let pg_app = PostgresApp {
                id: app.id,
                url: app.url.to_string(),
                secret_id: "".to_string(),
                meta_id: app.meta.id,
            };
            
            { // block is required because of connection release            
                let connection = get_connection().get()?;
                diesel::update(apps)
                    .set(&pg_app)
                    .execute(&connection)?;
            }
    
            Ok(())
        }
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

        // delete residual data from the app
        SECRET_REPO.delete(&app.secret)?;
        META_REPO.delete(&app.meta)?;

        // in order to avoid new sessions to be created while removing the current
        // ones it is required to purge sessions once the application itself has 
        // been removed from the system

        // delete all sessions related to the provided app
        SESS_REPO.delete_all_by_app(&app.url)?;

        // there cannot remain any directory of any user for the provided app
        DIR_REPO.delete_all_by_app(app.id)?;
        
        Ok(())
    }
}