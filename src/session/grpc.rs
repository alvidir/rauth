use super::application::SessionApplication;
use super::error::Error;
use crate::grpc;
use crate::mfa::service::MfaService;
use crate::secret::application::SecretRepository;
use crate::token::service::TokenService;
use crate::user::application::UserRepository;
use tonic::metadata::errors::InvalidMetadataValue;
use tonic::{Request, Response, Status};

// Import the generated rust code into module
mod proto {
    tonic::include_proto!("session");
}

// Proto generated server traits
use proto::session_server::Session;
pub use proto::session_server::SessionServer;

// Proto message structs
use proto::{Empty, LoginRequest};

impl From<Error> for Status {
    fn from(error: Error) -> Status {
        // match error {
        //     Error::NotAnEmail => Status::invalid_argument("email"),
        //     Error::NotAPassword => Status::invalid_argument("password"),
        //     Error::NotFound => Status::not_found("user"),
        //     Error::Unknown => Status::unknown(""),
        // }

        Status::unknown("")
    }
}

pub struct SessionGrpcService<U, S, T, F> {
    pub session_app: SessionApplication<U, S, T, F>,
    pub jwt_header: &'static str,
}

#[tonic::async_trait]
impl<U, S, T, F> Session for SessionGrpcService<U, S, T, F>
where
    U: 'static + UserRepository + Sync + Send,
    S: 'static + SecretRepository + Sync + Send,
    T: 'static + TokenService + Sync + Send,
    F: 'static + MfaService + Sync + Send,
{
    #[instrument(skip(self))]
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<Empty>, Status> {
        let msg_ref = request.into_inner();
        let token = self
            .session_app
            .login(&msg_ref.identifier, &msg_ref.password, &msg_ref.otp)
            .await
            .map_err(|err| Status::aborted(err.to_string()))?;

        let mut res = Response::new(Empty {});
        let token = token
            .as_ref()
            .parse()
            .map_err(|err: InvalidMetadataValue| {
                error!(error = err.to_string(), "parsing token to header");
                Into::<Status>::into(Error::Unknown)
            })?;

        res.metadata_mut().append(self.jwt_header, token);
        Ok(res)
    }

    #[instrument(skip(self))]
    async fn logout(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        let token = grpc::get_header(&request, self.jwt_header)?;
        if let Err(err) = self.session_app.logout(&token).await {
            return Err(Status::aborted(err.to_string()));
        }

        Ok(Response::new(Empty {}))
    }
}
