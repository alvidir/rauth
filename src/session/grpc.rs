use super::application::SessionApplication;
use crate::cache::Cache;
use crate::secret::application::SecretRepository;
use crate::user::application::UserRepository;
use crate::{grpc, result::Error};
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

pub struct SessionGrpcService<
    U: UserRepository + Sync + Send,
    S: SecretRepository + Sync + Send,
    C: Cache + Sync + Send,
> {
    pub session_app: SessionApplication<'static, U, S, C>,
    pub jwt_header: &'static str,
}

#[tonic::async_trait]
impl<
        U: 'static + UserRepository + Sync + Send,
        S: 'static + SecretRepository + Sync + Send,
        C: 'static + Cache + Sync + Send,
    > Session for SessionGrpcService<U, S, C>
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
