use std::error::Error;
use tonic::{Request, Response, Status};
use diesel::NotFound;

use crate::diesel::prelude::*;
use crate::postgres::*;
use crate::schema::users;
use crate::schema::users::dsl::*;
use crate::secret::framework::MongoSecretRepository;
use crate::secret::domain::SecretRepository;
use crate::metadata::framework::PostgresMetadataRepository;
use crate::metadata::domain::MetadataRepository;
use crate::session::framework::InMemorySessionRepository;
use crate::session::domain::SessionRepository;
use crate::directory::framework::MongoDirectoryRepository;
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

pub struct UserServiceImplementation {
    user_repo: &'static PostgresUserRepository,
    meta_repo: &'static PostgresMetadataRepository,
}

impl UserServiceImplementation {
    pub fn new(user_repo: &'static PostgresUserRepository,
               meta_repo: &'static PostgresMetadataRepository) -> Self {
        UserServiceImplementation {
            user_repo: user_repo,
            meta_repo: meta_repo,
        }
    }
}

#[tonic::async_trait]
impl UserService for UserServiceImplementation {
    async fn signup(&self, request: Request<SignupRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();

        match super::application::user_signup(&self.user_repo,
                                                          &self.meta_repo,
                                                          &msg_ref.email,
                                                          &msg_ref.pwd) {

            Err(err) => Err(Status::aborted(err.to_string())),
            Ok(_) => Ok(Response::new(())),
        }
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();

        match super::application::user_delete(&self.user_repo,
                                                &msg_ref.ident,
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

pub struct PostgresUserRepository {
    secret_repo: &'static MongoSecretRepository,
    dir_repo: &'static MongoDirectoryRepository,
    meta_repo: &'static PostgresMetadataRepository,
    sess_repo: &'static InMemorySessionRepository,
}

impl PostgresUserRepository {
    pub fn new(secret_repo: &'static MongoSecretRepository,
                sess_repo: &'static InMemorySessionRepository,
                dir_repo: &'static MongoDirectoryRepository,
                meta_repo: &'static PostgresMetadataRepository) -> Self {
        PostgresUserRepository {
            secret_repo: secret_repo,
            meta_repo: meta_repo,
            sess_repo: sess_repo,
            dir_repo: dir_repo,
        }
    }
}

impl UserRepository for &PostgresUserRepository {
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
            let secret = self.secret_repo.find(secr_id)?;
            secret_opt = Some(secret);
        }

        let meta = self.meta_repo.find(results[0].meta_id)?;

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
                secret_id: if let Some(secret) = &user.secret {Some(&secret.id)} else {None},
                meta_id: user.meta.id,
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
                email: user.email.clone(),
                password: user.password.clone(),
                verified: user.verified,
                secret_id: if let Some(secret) = &user.secret {Some(secret.id.clone())} else {None},
                meta_id: user.meta.id,
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

        // delete residual data from the user
        self.meta_repo.delete(&user.meta)?;
        if let Some(secret) = &user.secret {
            self.secret_repo.delete(secret)?;
        }

        // if the user have logged in, the session must be removed
        if let Ok(sess_arc) = self.sess_repo.find_by_email(&user.email) {
            let sess = sess_arc.read().unwrap(); // may panic if the lock was poisoned
            self.sess_repo.delete(&sess)?;
        }

        // there cannot remain any directory of any app for the provided user
        self.dir_repo.delete_all_by_user(user.id)?;
        Ok(())
    }
}