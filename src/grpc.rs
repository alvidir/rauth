//! gRPC utilities for managing request's headers.

use crate::base64::B64_CUSTOM_ENGINE;
use base64::Engine;
use tonic::{Request, Status};

use crate::result::Error;

/// Given a gPRC request, returns the value of the provided header's key if any, otherwise an error
/// is returned.
pub fn get_header<T>(req: &Request<T>, header: &str) -> Result<String, Status> {
    let data = req
        .metadata()
        .get(header)
        .ok_or_else(|| Status::aborted(Error::NotFound))
        .map(|data| data.to_str())?;

    data.map(|data| data.to_string()).map_err(|err| {
        warn!(
            "{} parsing header data to str: {}",
            Error::InvalidHeader,
            err
        );
        Status::aborted(Error::InvalidHeader)
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
        Status::unknown(Error::InvalidHeader)
    })?;

    let header = String::from_utf8(header).map_err(|err| {
        warn!("{} parsing header to str: {}", Error::InvalidHeader, err);
        Status::unknown(Error::InvalidHeader)
    })?;

    Ok(header)
}
