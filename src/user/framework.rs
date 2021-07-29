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
}

impl UserServiceImplementation {
    pub fn new(user_repo: &'static PostgresUserRepository,
               sess_repo: &'static InMemorySessionRepository,
               meta_repo: &'static PostgresMetadataRepository) -> Self {
        UserServiceImplementation {
            user_repo: user_repo,
            meta_repo: meta_repo,
            sess_repo: sess_repo,
        }
    }
}

impl UserServiceImplementation {
    fn send_verification_email(to: &str) {
        const SUBJECT: &str = "Verification email";
        const HTML: &str = "<h1>Click here in order to verificate your email</h1>";
        
        if let Err(err) = smtp::send_email(to, SUBJECT, HTML) {
            println!("got error {}\nwhile sending verification email to {}", err, to);
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
    pub password: String,
    pub secret_id: Option<String>,
    pub meta_id: i32,
}

#[derive(Insertable)]
#[derive(Clone)]
#[table_name = "users"]
struct NewPostgresUser<'a> {
    pub email: &'a str,
    pub password: &'a str,
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
            password: results[0].password.clone(),
            secret: secret_opt,
            meta: meta,
        })
    }

    fn save(&self, user: &mut User) -> Result<(), Box<dyn Error>> {
        if user.id == 0 { // create user
            let new_user = NewPostgresUser {
                email: &user.email,
                password: &user.password,
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
                password: user.password.clone(),
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