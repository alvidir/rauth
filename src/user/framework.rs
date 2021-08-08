use std::error::Error;
use tonic::{Request, Response, Status};
use diesel::NotFound;

use crate::diesel::prelude::*;
use crate::postgres::*;
use crate::schema::users;
use crate::schema::users::dsl::*;
use crate::metadata::get_repository as get_meta_repository;
use crate::secret::get_repository as get_secret_repository;
use super::domain::{User, UserRepository};

// Import the generated rust code into module
mod proto {
    tonic::include_proto!("user");
}

// Proto generated server traits
use proto::user_service_server::UserService;
pub use proto::user_service_server::UserServiceServer;

// Proto message structs
use proto::{SignupRequest, DeleteRequest };

pub struct UserServiceImplementation;

#[tonic::async_trait]
impl UserService for UserServiceImplementation {
    async fn signup(&self, request: Request<SignupRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();

        match super::application::user_signup(&msg_ref.email,
                                              &msg_ref.pwd) {

            Err(err) => Err(Status::aborted(err.to_string())),
            Ok(_) => Ok(Response::new(())),
        }
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();

        match super::application::user_delete(&msg_ref.ident,
                                              &msg_ref.pwd,
                                              &msg_ref.totp) {

            Err(err) => Err(Status::aborted(err.to_string())),
            Ok(()) => Ok(Response::new(())),
        }
    }
}

#[derive(Queryable, Insertable, Associations)]
#[derive(Identifiable)]
#[derive(AsChangeset)]
#[derive(Clone)]
#[table_name = "users"]
struct PostgresUser {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub verified: bool,
    pub secret_id: Option<String>,
    pub meta_id: i32,
}

#[derive(Insertable)]
#[derive(Clone)]
#[table_name = "users"]
struct NewPostgresUser<'a> {
    pub email: &'a str,
    pub password: &'a str,
    pub verified: bool,
    pub secret_id: Option<&'a str>,
    pub meta_id: i32,
}

pub struct PostgresUserRepository;

impl UserRepository for PostgresUserRepository {
    fn find(&self, target: &str) -> Result<User, Box<dyn Error>>  {
        use crate::schema::users::dsl::*;
        
        let results = { // block is required because of connection release
            let connection = get_connection().get()?;
            users.filter(email.eq(target))
                 .load::<PostgresUser>(&connection)?
        };
    
        if results.len() == 0 {
            return Err(Box::new(NotFound));
        }

        let mut secret_opt = None;
        if let Some(secr_id) = &results[0].secret_id {
            let secret = get_secret_repository().find(secr_id)?;
            secret_opt = Some(secret);
        }

        let meta = get_meta_repository().find(results[0].meta_id)?;

        Ok(User{
            id: results[0].id,
            email: results[0].email.clone(),
            password: results[0].password.clone(),
            verified: results[0].verified,
            secret: secret_opt,
            meta: meta,
        })
    }

    fn save(&self, user: &mut User) -> Result<(), Box<dyn Error>> {
        if user.id == 0 { // create user
            let new_user = NewPostgresUser {
                email: &user.email,
                password: &user.password,
                verified: user.verified,
                secret_id: if let Some(secret) = &user.secret {Some(secret.get_id())} else {None},
                meta_id: user.meta.get_id(),
            };
    
            let result = { // block is required because of connection release
                let connection = get_connection().get()?;
                diesel::insert_into(users::table)
                    .values(&new_user)
                    .get_result::<PostgresUser>(&connection)?
            };
    
            user.id = result.id;
            Ok(())

        } else { // update user
            let pg_user = PostgresUser {
                id: user.id,
                email: user.email.to_string(),
                password: user.password.clone(),
                verified: user.verified,
                secret_id: if let Some(secret) = &user.secret {Some(secret.get_id().to_string())} else {None},
                meta_id: user.meta.get_id(),
            };
            
            { // block is required because of connection release            
                let connection = get_connection().get()?;
                diesel::update(users)
                    .set(&pg_user)
                    .execute(&connection)?;
            }
    
            Ok(())
        }
    }

    fn delete(&self, user: &User) -> Result<(), Box<dyn Error>> {
        { // block is required because of connection release
            let connection = get_connection().get()?;
            let _result = diesel::delete(
                users.filter(
                    id.eq(user.id)
                )
            ).execute(&connection)?;
        }

        Ok(())
    }
}