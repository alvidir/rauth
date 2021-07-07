use std::error::Error;
use tonic::{Request, Response, Status};
use diesel::NotFound;

use crate::diesel::prelude::*;
use crate::postgres::*;

extern crate diesel;

// Import the generated rust code into module
mod proto {
    tonic::include_proto!("user");
}

// Proto generated server traits
use proto::user_service_server::UserService;
pub use proto::user_service_server::UserServiceServer;

// Proto message structs
use proto::{LoginRequest, SignupRequest, LoginResponse, DeleteRequest };

#[derive(Default)]
pub struct UserServiceImplementation {}

use super::domain::User;

#[tonic::async_trait]
impl UserService for UserServiceImplementation {
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<LoginResponse>, Status> {
        let msg_ref = request.into_inner();
        Err(Status::unimplemented(""))
    }

    async fn logout(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();
        Err(Status::unimplemented(""))
    }

    async fn signup(&self, request: Request<SignupRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();
        Err(Status::unimplemented(""))
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();
        Err(Status::unimplemented(""))
    }
}

pub struct PostgresUserRepository {}

impl PostgresUserRepository {
    pub fn find(target: &str) -> Result<User, Box<dyn Error>>  {
        use crate::schema::users::dsl::*;
        
        let results = { // block is required because of connection release
            let connection = open_stream().get()?;
            users.filter(email.eq(target))
                 .load::<User>(&connection)?
        };
    
        if results.len() > 0 {
            Ok(results[0].clone())
        } else {
            Err(Box::new(NotFound))
        }
    }
}