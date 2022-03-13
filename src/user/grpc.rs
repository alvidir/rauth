use tonic::{Request, Response, Status};
use crate::{security, constants, grpc};
use crate::user::application::{UserRepository, UserApplication};
use crate::secret::application::SecretRepository;
use crate::smtp::Mailer;
use crate::session::application::TokenRepository;

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
}

#[tonic::async_trait]
impl<
    U: 'static + UserRepository + Sync + Send,
    E: 'static + SecretRepository + Sync + Send,
    S: 'static + TokenRepository + Sync + Send,
    M: 'static + Mailer + Sync + Send,
    > User for UserImplementation<U, E, S, M> {
    async fn signup(&self, request: Request<SignupRequest>) -> Result<Response<Empty>, Status> {
        if request.metadata().get(self.jwt_header).is_some() {
            let token = grpc::get_encoded_header(&request, self.jwt_header)?;
            let token = self.user_app.secure_signup(&token, self.jwt_public, self.jwt_secret)
                .map_err(|err| Status::aborted(err.to_string()))?;
            
            let mut res = Response::new(Empty{});
            let token = token.parse()
                .map_err(|err| {
                    error!("{} parsing token to header: {}", constants::ERR_UNKNOWN, err);
                    Status::unknown(constants::ERR_UNKNOWN)
                })?;

            res.metadata_mut().append(self.jwt_header, token);
            return Ok(res);
        }
        
        let msg_ref = request.into_inner();
        let shadowed_pwd = security::shadow(&msg_ref.pwd, self.pwd_sufix);

        self.user_app.verify_signup_email(&msg_ref.email, &shadowed_pwd, self.jwt_secret)
            .map_err(|err| Status::aborted(err.to_string()))?;
        
        Err(Status::failed_precondition(constants::ERR_NOT_AVAILABLE))
    }

    async fn reset(&self, request: Request<ResetRequest>) -> Result<Response<Empty>, Status> {
        if request.metadata().get(self.jwt_header).is_some() {
            let token = grpc::get_encoded_header(&request, self.jwt_header)?;
            let msg_ref = request.into_inner();
            let shadowed_pwd = security::shadow(&msg_ref.pwd, self.pwd_sufix);
            return self.user_app.secure_reset(&shadowed_pwd, &msg_ref.totp, &token, self.jwt_public)
                .map(|_| Response::new(Empty{}))
                .map_err(|err| Status::aborted(err.to_string()));
        }

        let msg_ref = request.into_inner();
        self.user_app.verify_reset_email(&msg_ref.email, self.jwt_secret)
            .map_err(|err| Status::aborted(err.to_string()))?;
        
        Err(Status::failed_precondition(constants::ERR_NOT_AVAILABLE))
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<Empty>, Status> {
        let token = grpc::get_encoded_header(&request, self.jwt_header)?;
        let msg_ref = request.into_inner();
        
        let shadowed_pwd = security::shadow(&msg_ref.pwd, self.pwd_sufix);
        self.user_app.secure_delete(&shadowed_pwd, &msg_ref.totp, &token, self.jwt_public)
            .map(|_| Response::new(Empty{}))
            .map_err(|err| Status::aborted(err.to_string()))
    }

    async fn totp(&self, request: Request<TotpRequest>) -> Result<Response<Empty>, Status> {
        let token = grpc::get_encoded_header(&request, self.jwt_header)?;
        let msg_ref = request.into_inner();
        let shadowed_pwd = security::shadow(&msg_ref.pwd, self.pwd_sufix);

        if msg_ref.action == TOTP_ACTION_ENABLE {
            let token = self.user_app.secure_enable_totp(&shadowed_pwd, &msg_ref.totp, &token, self.jwt_public)
                .map_err(|err| Status::aborted(err.to_string()))?;

            let mut response = Response::new(Empty{});
            if let Some(token) = token {
                let token = token.parse().map_err(|err| {
                    error!("{} parsing str to metadata: {}", constants::ERR_UNKNOWN, err);
                    Status::aborted(constants::ERR_UNKNOWN.to_string())
                })?;

                response.metadata_mut().insert(self.totp_header, token);
            }
        }

        if msg_ref.action == TOTP_ACTION_DISABLE {
            return self.user_app.secure_disable_totp(&shadowed_pwd, &msg_ref.totp, &token, self.jwt_public)
                .map(|_| Response::new(Empty{}))
                .map_err(|err| Status::unknown(err.to_string()));
        }

        Err(Status::invalid_argument(constants::ERR_NOT_AVAILABLE))
    }
}