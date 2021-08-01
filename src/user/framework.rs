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
use crate::directory::framework::MongoDirectoryRepository;
use crate::smtp;
use crate::security;
use crate::constants::ERR_NOT_FOUND;
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
    sess_repo: &'static InMemorySessionRepository,
    dir_repo: &'static MongoDirectoryRepository,
    secret_repo: &'static MongoSecretRepository,
}

impl UserServiceImplementation {
    pub fn new(user_repo: &'static PostgresUserRepository,
               sess_repo: &'static InMemorySessionRepository,
               dir_repo: &'static MongoDirectoryRepository,
               secret_repo: &'static MongoSecretRepository,
               meta_repo: &'static PostgresMetadataRepository) -> Self {
        UserServiceImplementation {
            user_repo: user_repo,
            meta_repo: meta_repo,
            dir_repo: dir_repo,
            secret_repo: secret_repo,
            sess_repo: sess_repo,
        }
    }
}

impl UserServiceImplementation {
    fn send_verification_email(to: &str) {
        const SUBJECT: &str = "Verification email";
        const HTML: &str = "<h1>Click here in order to verificate your email</h1>";
        
        if let Err(err) = smtp::send_email(to, SUBJECT, HTML) {
            info!("got error {}\nwhile sending verification email to {}", err, to);
        }

    }
}

#[tonic::async_trait]
impl UserService for UserServiceImplementation {
    async fn signup(&self, request: Request<SignupRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();

        if let Err(err) = super::application::user_signup(&self.user_repo,
                                                          &self.meta_repo,
                                                          &msg_ref.email,
                                                          &msg_ref.pwd) {

            return Err(Status::aborted(err.to_string()));
        }

        // the email is required in order to verify the identity of the user
        UserServiceImplementation::send_verification_email(&msg_ref.email);
        Ok(Response::new(()))
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();

        match self.user_repo.find(&msg_ref.ident) {
            Err(_) => {
                // in order to give no clue about if the error was about the email or password
                // both cases must provide the same kind of error
                return Err(Status::not_found(ERR_NOT_FOUND))
            },

            Ok(user) => {
                if !user.match_password(&msg_ref.pwd) {
                    // same error as if the user was not found
                    return Err(Status::not_found(ERR_NOT_FOUND));
                }

                // if, and only if, the user has activated the 2fa
                if let Some(secret) = user.secret {
                    let data = secret.get_data();
                    if let Err(err) = security::verify_totp_password(data, &msg_ref.pwd) {
                        // in order to make the application know a valid TOTP is required
                        return Err(Status::unauthenticated(err.to_string()));
                    }
                }
            }
        };

        match super::application::user_delete(&self.user_repo,
                                              &self.sess_repo,
                                              &self.dir_repo,
                                              &self.secret_repo,
                                              &self.meta_repo,
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

        let meta = self.metadata_repo.find(results[0].meta_id)?;

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

        Ok(())
    }
}