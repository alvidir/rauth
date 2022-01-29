use tonic::{Request, Response, Status};
use crate::security;
use crate::constants;
use crate::user::application::{UserRepository, UserApplication};
use crate::secret::application::SecretRepository;
use crate::grpc;

// Import the generated rust code into module
mod proto {
    tonic::include_proto!("user");
}

// Proto generated server traits
use proto::user_server::User;
pub use proto::user_server::UserServer;

// Proto message structs
use proto::{SignupRequest, ResetPasswordRequest, DeleteRequest, TotpRequest, Empty};

pub struct UserImplementation<
    U: UserRepository + Sync + Send,
    E:  SecretRepository + Sync + Send
    > {
    pub user_app: UserApplication<U, E>,
    pub rsa_secret: &'static [u8],
    pub jwt_public: &'static [u8],
    pub jwt_header: &'static str,
    pub allow_unverified: bool,
}

#[tonic::async_trait]
impl<
    U: 'static + UserRepository + Sync + Send,
    E: 'static + SecretRepository + Sync + Send
    > User for UserImplementation<U, E> {
    async fn signup(&self, request: Request<SignupRequest>) -> Result<Response<Empty>, Status> {
        if request.metadata().get(self.jwt_header).is_none() {
            if !self.allow_unverified {
                return Err(Status::failed_precondition(constants::ERR_UNVERIFIED))
            }

            let msg_ref = request.into_inner();
            match self.user_app.signup(&msg_ref.email, &msg_ref.pwd) {
                Err(err) => return Err(Status::aborted(err.to_string())),
                Ok(_) => return Ok(Response::new(Empty{})),
            };
        }
            
        let secure_jwt = grpc::get_header(&request, self.jwt_header)?;
        let jwt = match security::decrypt(self.rsa_secret, secure_jwt.as_bytes()) {
            Ok(token) => String::from_utf8(token).map_err(|err| {
                warn!("{}: {}", constants::ERR_PARSE_HEADER, err);
                Status::unknown(constants::ERR_PARSE_HEADER)
            })?,

            Err(err) => {
                warn!("{}: {}", constants::ERR_DECRYPT_TOKEN, err);
                return Err(Status::invalid_argument(constants::ERR_DECRYPT_TOKEN))
            },
        };

        let msg_ref = request.into_inner();
        match self.user_app.secure_signup(&msg_ref.email, &msg_ref.pwd, &jwt, self.jwt_public) {
            Err(err) => Err(Status::aborted(err.to_string())),
            Ok(_) => Ok(Response::new(Empty{})),
        }
    }

    async fn reset_password(&self, _: Request<ResetPasswordRequest>) -> Result<Response<Empty>, Status> {
        return Err(Status::unimplemented("not implemented".to_string()));
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<Empty>, Status> {
        let token = grpc::get_header(&request, self.jwt_header)?;
        let msg_ref = request.into_inner();
        
        match self.user_app.secure_delete(&msg_ref.pwd, &msg_ref.totp, &token, self.jwt_public) {
            Err(err) => Err(Status::aborted(err.to_string())),
            Ok(()) => Ok(Response::new(Empty{})),
        }
    }

    async fn totp(&self, request: Request<TotpRequest>) -> Result<Response<Empty>, Status> {
        let token = grpc::get_header(&request, self.jwt_header)?;
        let msg_ref = request.into_inner();

        if msg_ref.action == 0 {
            match self.user_app.secure_enable_totp(&msg_ref.pwd, &msg_ref.totp, &token, self.jwt_public) {
                Err(err) => return Err(Status::unknown(err.to_string())),
                Ok(token) => {
                    let mut response = Response::new(Empty{});
                    response.metadata_mut().insert(self.jwt_header, token.parse().unwrap());
                    return Ok(response);
                }
            }
        }

        if msg_ref.action == 1 {
            match self.user_app.secure_disable_totp(&msg_ref.pwd, &msg_ref.totp, &token, self.jwt_public) {
                Ok(_) => return Ok(Response::new(Empty{})),
                Err(err) => return Err(Status::unknown(err.to_string())),
            }
        }

        Err(Status::invalid_argument(constants::ERR_INVALID_OPTION))
    }
}