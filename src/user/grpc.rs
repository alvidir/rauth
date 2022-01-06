use std::error::Error;
use std::time::SystemTime;
use tonic::{Request, Response, Status};
use diesel::NotFound;
use diesel::result::Error as PgError;

use crate::diesel::prelude::*;
use crate::postgres::*;
use crate::schema::users;
use crate::schema::users::dsl::*;
use crate::metadata::{
    get_repository as get_meta_repository,
    framework::PostgresMetadataRepository,
};

use super::domain::{User, UserRepository};
use super::application::TfaActions;

// Import the generated rust code into module
mod proto {
    tonic::include_proto!("user");
}

// Proto generated server traits
use proto::user_service_server::UserService;
pub use proto::user_service_server::UserServiceServer;

// Proto message structs
use proto::{SignupRequest, Secure, TotpRequest};

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

    async fn delete(&self, request: Request<Secure>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();

        match super::application::user_delete(&msg_ref.ident,
                                              &msg_ref.pwd,
                                              &msg_ref.totp) {

            Err(err) => Err(Status::aborted(err.to_string())),
            Ok(()) => Ok(Response::new(())),
        }
    }

    async fn totp(&self, request: Request<TotpRequest>) -> Result<Response<()>, Status> {
        if let None = request.metadata().get("token") {
            return Err(Status::failed_precondition("token required"));
        };

        let token = match request.metadata().get("token")
            .unwrap() // this line will not fail due to the previous check of None 
            .to_str() {
            Err(err) => return Err(Status::aborted(err.to_string())),
            Ok(token) => token.to_string(),
        };

        let msg_ref = request.into_inner();
        let action = match msg_ref.action {
            0 => TfaActions::ENABLE,
            1 => TfaActions::DISABLE,
            _ => return Err(Status::invalid_argument("wrong action")),
        };

        match super::application::user_two_factor_authenticator(&token,
                                                                &msg_ref.pwd,
                                                                &msg_ref.totp,
                                                                action) {
            Err(err) => Err(Status::aborted(err.to_string())),
            Ok(uri) => Ok(Response::new(
                TfaResponse{
                    uri: uri,
                }
            )),
        }
    }
}