use tonic::{Request, Response, Status};
use crate::security;
use crate::constants;
use crate::user::application::{UserRepository, UserApplication};
use crate::secret::application::SecretRepository;
use crate::session::{
    grpc::get_session_token,
    domain::VerificationToken
};

// Import the generated rust code into module
mod proto {
    tonic::include_proto!("user");
}

// Proto generated server traits
use proto::user_service_server::UserService;
pub use proto::user_service_server::UserServiceServer;

// Proto message structs
use proto::{SignupRequest, DeleteRequest, TotpRequest};

pub struct UserServiceImplementation<
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
    > UserService for UserServiceImplementation<U, E> {
    async fn signup(&self, request: Request<SignupRequest>) -> Result<Response<()>, Status> {
        if request.metadata().get(self.jwt_header).is_none() {
            if !self.allow_unverified {
                return Err(Status::failed_precondition(constants::ERR_UNVERIFIED))
            }

            let msg_ref = request.into_inner();
            let shadowed_pwd = security::shadow(&msg_ref.pwd, constants::PWD_SUFIX);
            match self.user_app.signup(&msg_ref.email, &shadowed_pwd) {
                Err(err) => return Err(Status::aborted(err.to_string())),
                Ok(_) => return Ok(Response::new(())),
            };
        }
    
        // this line will not fail due to the previous check of None 
        let secure_token = request.metadata().get(self.jwt_header).unwrap().to_str()
            .map_err(|err| {
                error!("{}: {}", constants::ERR_PARSE_TOKEN, err);
                Status::aborted(constants::ERR_PARSE_TOKEN)
            })?;

        let token = match security::decrypt(self.rsa_secret, secure_token.as_bytes()) {
            Ok(token) => String::from_utf8(token).map_err(|err| {
                warn!("{}: {}", constants::ERR_PARSE_TOKEN, err);
                Status::unknown(constants::ERR_PARSE_TOKEN)
            })?,

            Err(err) => {
                warn!("{}: {}", constants::ERR_DECRYPT_TOKEN, err);
                return Err(Status::invalid_argument(constants::ERR_DECRYPT_TOKEN))
            },
        };

        let claims = security::verify_jwt::<VerificationToken>(&self.jwt_public, &token)
            .map_err(|err| {
                warn!("{}: {}", constants::ERR_VERIFY_TOKEN, err);
                Status::aborted(constants::ERR_VERIFY_TOKEN)
            })?;

        let msg_ref = request.into_inner();
        if claims.sub.is_some() && claims.pwd.is_none() {
            // this line will not fail due to the previous check of Some
            let shadowed_pwd = security::shadow(&msg_ref.pwd, constants::PWD_SUFIX);
            match self.user_app.signup(&claims.sub.unwrap(), &shadowed_pwd) {
                Err(err) => return Err(Status::aborted(err.to_string())),
                Ok(_) => return Ok(Response::new(())),
            };
        }

        if claims.sub.is_some() && claims.pwd.is_some() {
            // this line will not fail due to the previous check of Some
            match self.user_app.signup(&claims.sub.unwrap(), &claims.pwd.unwrap()) {
                Err(err) => return Err(Status::aborted(err.to_string())),
                Ok(_) => return Ok(Response::new(())),
            };
        }

        Err(Status::invalid_argument(constants::ERR_MISSING_DATA))
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<()>, Status> {
        let claims =  get_session_token(request.metadata(), self.jwt_public, self.jwt_header)
            .map_err(|err| {
                warn!("{}: {}", constants::ERR_VERIFY_TOKEN, err);
                Status::unauthenticated(constants::ERR_VERIFY_TOKEN)
            })?;

        let msg_ref = request.into_inner();
        match self.user_app.delete(claims.sub, &msg_ref.pwd, &msg_ref.totp) {
            Err(err) => Err(Status::aborted(err.to_string())),
            Ok(()) => Ok(Response::new(())),
        }
    }

    async fn totp(&self, request: Request<TotpRequest>) -> Result<Response<()>, Status> {
        let claims =  get_session_token(request.metadata(), self.jwt_public, self.jwt_header)
            .map_err(|err| {
                warn!("{}: {}", constants::ERR_VERIFY_TOKEN, err);
                Status::unauthenticated(constants::ERR_VERIFY_TOKEN)
            })?;

        let msg_ref = request.into_inner(); 
        if msg_ref.action == 0 {
            match self.user_app.enable_totp(claims.sub, &msg_ref.pwd, &msg_ref.totp) {
                Err(err) => return Err(Status::unknown(err.to_string())),
                Ok(token) => {
                    let mut response = Response::new(());
                    response.metadata_mut().insert(self.jwt_header, token.parse().unwrap());
                    return Ok(response);
                }
            }
        }

        if msg_ref.action == 1 {
            match self.user_app.disable_totp(claims.sub, &msg_ref.pwd, &msg_ref.totp) {
                Ok(_) => return Ok(Response::new(())),
                Err(err) => return Err(Status::unknown(err.to_string())),
            }
        }

        Err(Status::invalid_argument(constants::ERR_INVALID_OPTION))
    }
}