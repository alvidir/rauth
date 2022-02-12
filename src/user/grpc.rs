use tonic::{Request, Response, Status};
use crate::security;
use crate::constants;
use crate::user::application::{UserRepository, UserApplication};
use crate::secret::application::SecretRepository;
use crate::smtp::Mailer;
use crate::session::{
    grpc::util::get_token,
    application::TokenRepository
};

const RESET_ACTION_PWD: i32 = 0;

const TOTP_ACTION_ENABLE: i32 = 0;
const TOTP_ACTION_DISABLE: i32 = 1;

// Import the generated rust code into module
mod proto {
    tonic::include_proto!("user");
}

// Proto generated server traits
use proto::user_server::User;
pub use proto::user_server::UserServer;

// Proto message structs
use proto::{SignupRequest, ResetRequest, DeleteRequest, TotpRequest, Empty};

pub struct UserImplementation<
    U: UserRepository + Sync + Send,
    E:  SecretRepository + Sync + Send,
    S: TokenRepository + Sync + Send,
    M: Mailer,
    > {
    pub user_app: UserApplication<U, E, S, M>,
    pub jwt_secret: &'static [u8],
    pub jwt_public: &'static [u8],
    pub jwt_header: &'static str,
    pub totp_header: &'static str,
    pub pwd_sufix: &'static str,
    pub allow_unverified: bool,
}

#[tonic::async_trait]
impl<
    U: 'static + UserRepository + Sync + Send,
    E: 'static + SecretRepository + Sync + Send,
    S: 'static + TokenRepository + Sync + Send,
    M: 'static + Mailer + Sync + Send,
    > User for UserImplementation<U, E, S, M> {
    async fn signup(&self, request: Request<SignupRequest>) -> Result<Response<Empty>, Status> {
        if request.metadata().get(self.jwt_header).is_none() {
            let msg_ref = request.into_inner();
            let shadowed_pwd = security::shadow(&msg_ref.pwd, self.pwd_sufix);

            if !self.allow_unverified {
                self.user_app.verify_signup_email(&msg_ref.email, &shadowed_pwd, self.jwt_secret)
                    .map_err(|err| {
                        error!("{}: {}", constants::ERR_SEND_EMAIL, err);
                        Status::aborted(constants::ERR_SEND_EMAIL)
                    })?;
                
                return Err(Status::failed_precondition(constants::ERR_UNVERIFIED))
            }

            match self.user_app.signup(&msg_ref.email, &shadowed_pwd) {
                Err(err) => return Err(Status::aborted(err.to_string())),
                Ok(_) => return Ok(Response::new(Empty{})),
            };
        }
        
        let token = get_token(&request, self.jwt_header)?;
        match self.user_app.secure_signup(&token, self.jwt_public) {
            Err(err) => Err(Status::aborted(err.to_string())),
            Ok(_) => Ok(Response::new(Empty{})),
        }
    }

    async fn reset(&self, request: Request<ResetRequest>) -> Result<Response<Empty>, Status> {
        if request.metadata().get(self.jwt_header).is_none() {
            // only 'reset password' requests may have no token
            let msg_ref = request.into_inner();
            if msg_ref.action != RESET_ACTION_PWD {
                return Err(Status::aborted(constants::ERR_UNAUTHORIZED));
            }

            self.user_app.verify_reset_pwd_email(&msg_ref.email, self.jwt_secret)
                .map_err(|err| {
                    error!("{}: {}", constants::ERR_SEND_EMAIL, err);
                    Status::aborted(constants::ERR_SEND_EMAIL)
                })?;
            
            return Err(Status::failed_precondition(constants::ERR_UNVERIFIED))
        }

        let token = get_token(&request, self.jwt_header)?;
        let msg_ref = request.into_inner();
        if msg_ref.action == RESET_ACTION_PWD {
            let shadowed_pwd = security::shadow(&msg_ref.pwd, self.pwd_sufix);
            match self.user_app.secure_reset_pwd(&shadowed_pwd, &msg_ref.totp, &token, self.jwt_public) {
                Err(err) => return Err(Status::aborted(err.to_string())),
                Ok(_) => return Ok(Response::new(Empty{})),
            }
        }

        Err(Status::invalid_argument(constants::ERR_INVALID_OPTION))
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<Empty>, Status> {
        let token = get_token(&request, self.jwt_header)?;
        let msg_ref = request.into_inner();
        
        let shadowed_pwd = security::shadow(&msg_ref.pwd, self.pwd_sufix);
        match self.user_app.secure_delete(&shadowed_pwd, &msg_ref.totp, &token, self.jwt_public) {
            Err(err) => Err(Status::aborted(err.to_string())),
            Ok(()) => Ok(Response::new(Empty{})),
        }
    }

    async fn totp(&self, request: Request<TotpRequest>) -> Result<Response<Empty>, Status> {
        let token = get_token(&request, self.jwt_header)?;
        let msg_ref = request.into_inner();
        let shadowed_pwd = security::shadow(&msg_ref.pwd, self.pwd_sufix);

        if msg_ref.action == TOTP_ACTION_ENABLE {
            match self.user_app.secure_enable_totp(&shadowed_pwd, &msg_ref.totp, &token, self.jwt_public) {
                Err(err) => return Err(Status::unknown(err.to_string())),
                Ok(token) => {
                    let mut response = Response::new(Empty{});
                    response.metadata_mut().insert(self.totp_header, token.parse().unwrap());
                    return Ok(response);
                }
            }
        }

        if msg_ref.action == TOTP_ACTION_DISABLE {
            match self.user_app.secure_disable_totp(&shadowed_pwd, &msg_ref.totp, &token, self.jwt_public) {
                Ok(_) => return Ok(Response::new(Empty{})),
                Err(err) => return Err(Status::unknown(err.to_string())),
            }
        }

        Err(Status::invalid_argument(constants::ERR_INVALID_OPTION))
    }
}