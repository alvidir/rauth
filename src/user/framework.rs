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
use crate::smtp;
use crate::security;

use super::domain::{User, UserRepository};
use super::application::EmailManager;

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
    sess_repo: &'static InMemorySessionRepository,
    email_manager: SMTPEmailManager
}

impl UserServiceImplementation {
    pub fn new(user_repo: &'static PostgresUserRepository,
               sess_repo: &'static InMemorySessionRepository,
               meta_repo: &'static PostgresMetadataRepository) -> Self {
        UserServiceImplementation {
            user_repo: user_repo,
            meta_repo: meta_repo,
            sess_repo: sess_repo,
            email_manager: SMTPEmailManager{}
        }
    }
}

#[tonic::async_trait]
impl UserService for UserServiceImplementation {
    async fn signup(&self, request: Request<SignupRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();

        match super::application::user_signup(Box::new(self.user_repo),
                                              Box::new(self.meta_repo),
                                              Box::new(self.email_manager),
                                              &msg_ref.email) {

            Err(err) => Err(Status::aborted(err.to_string())),
            Ok(()) => Ok(Response::new(())),
        }
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();

        let user_search = self.user_repo.find(&msg_ref.ident);
        if let Err(err) = user_search {
            return Err(Status::not_found(err.to_string()));
        } 

        let user = user_search.unwrap();
        if let Some(secret) = &user.secret {
            // the provided password must be the same as the TOTP obtained from the secret
            let key = secret.get_data();
            if let Err(err) = security::verify_totp_password(key, &msg_ref.pwd) {
                return Err(Status::unauthenticated(err.to_string()));
            }
        }

        match super::application::user_delete(Box::new(self.user_repo),
                                              Box::new(self.sess_repo),
                                              &msg_ref.ident) {

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
    pub secret_id: Option<String>,
    pub meta_id: i32,
}

#[derive(Insertable)]
#[derive(Clone)]
#[table_name = "users"]
struct NewPostgresUser<'a> {
    pub email: &'a str,
    pub secret_id: Option<&'a str>,
    pub meta_id: i32,
}

pub struct PostgresUserRepository {
    secret_repo: &'static MongoSecretRepository,
    metadata_repo: &'static PostgresMetadataRepository,
}

impl PostgresUserRepository {
    pub fn new(secret_repo: &'static MongoSecretRepository,
               meta_repo: &'static PostgresMetadataRepository) -> Self {
        PostgresUserRepository {
            secret_repo: secret_repo,
            metadata_repo: meta_repo,
        }
    }
}

impl UserRepository for &PostgresUserRepository {
    fn find(&self, target: &str) -> Result<User, Box<dyn Error>>  {
        use crate::schema::users::dsl::*;
        
        let results = { // block is required because of connection release
            let connection = open_stream().get()?;
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

        let meta = self.metadata_repo.find(results[0].meta_id)?;

        Ok(User{
            id: results[0].id,
            email: results[0].email.clone(),
            secret: secret_opt,
            meta: meta,
        })
    }

    fn save(&self, user: &mut User) -> Result<(), Box<dyn Error>> {
        if user.id == 0 { // create user
            let new_user = NewPostgresUser {
                email: &user.email,
                secret_id: if let Some(secret) = &user.secret {Some(&secret.id)} else {None},
                meta_id: user.meta.id,
            };
    
            let result = { // block is required because of connection release
                let connection = open_stream().get()?;
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
                secret_id: if let Some(secret) = &user.secret {Some(secret.id.clone())} else {None},
                meta_id: user.meta.id,
            };
            
            { // block is required because of connection release            
                let connection = open_stream().get()?;
                diesel::update(users)
                    .set(&pg_user)
                    .execute(&connection)?;
            }
    
            Ok(())
        }
    }

    fn delete(&self, user: &User) -> Result<(), Box<dyn Error>> {
        { // block is required because of connection release
            let connection = open_stream().get()?;
            let _result = diesel::delete(
                users.filter(
                    id.eq(user.id)
                )
            ).execute(&connection)?;
        }

        Ok(())
    }
}

#[derive(Copy, Clone)]
pub struct SMTPEmailManager {}

impl EmailManager for SMTPEmailManager {
    fn send_verification_email(&self, to: &str) -> Result<(), Box<dyn Error>> {
        smtp::send_email(to, "Verification email", "<h1>Click here in order to verificate your email</h1>")
    }
}