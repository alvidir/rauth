//! gRPC utilities for managing request's headers.

use tonic::{Request, Status};

use crate::result::Error;

impl From<Error> for Status {
    fn from(value: Error) -> Self {
        match value {
            Error::Unknown => Status::unknown(value),
            Error::NotFound => Status::not_found(value),
            Error::NotAvailable => Status::unavailable(value),
            Error::Unauthorized => Status::permission_denied(value),
            Error::InvalidToken | Error::InvalidFormat | Error::InvalidHeader => {
                Status::invalid_argument(value)
            }
            Error::WrongCredentials => Status::unauthenticated(value),
            Error::RegexNotMatch => Status::failed_precondition(value),
        }
    }
}

/// Given a gPRC request, returns the value of the provided header's key if any, otherwise an error
/// is returned.
pub fn get_header<T>(req: &Request<T>, header: &str) -> Result<String, Error> {
    let data = req
        .metadata()
        .get(header)
        .ok_or(Error::NotFound)
        .map(|data| data.to_str())?;

    data.map(|data| data.to_string()).map_err(|err| {
        warn!(error = err.to_string(), "parsing header data to str",);
        Error::InvalidHeader
    })
}
