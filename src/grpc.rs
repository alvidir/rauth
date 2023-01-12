//! gRPC utilities for managing request's headers.

use crate::base64::B64_CUSTOM_ENGINE;
use base64::Engine;
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
pub fn get_header<T>(req: &Request<T>, header: &str) -> Result<String, Status> {
    let data = req
        .metadata()
        .get(header)
        .ok_or_else(|| Into::<Status>::into(Error::NotFound))
        .map(|data| data.to_str())?;

    data.map(|data| data.to_string()).map_err(|err| {
        warn!(
            "{} parsing header data to str: {}",
            Error::InvalidHeader,
            err
        );
        Error::InvalidHeader.into()
    })
}

/// Given a gPRC request, returns the base64 decoded value of the provided header's key if any, otherwise
/// an error is returned.
pub fn get_encoded_header<T>(request: &Request<T>, header: &str) -> Result<String, Status> {
    let header = get_header(request, header)?;
    let header = B64_CUSTOM_ENGINE.decode(header).map_err(|err| {
        warn!(
            "{} decoding header from base64: {}",
            Error::InvalidHeader,
            err
        );
        Into::<Status>::into(Error::InvalidHeader)
    })?;

    let header = String::from_utf8(header).map_err(|err| {
        warn!("{} parsing header to str: {}", Error::InvalidHeader, err);
        Into::<Status>::into(Error::InvalidHeader)
    })?;

    Ok(header)
}
