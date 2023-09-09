//! gRPC utilities for managing request's headers.

use crate::on_error;
use tonic::Request;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Metadata(#[from] tonic::metadata::errors::ToStrError),
}

/// Given a gPRC request, returns the value of the provided header's key if any, otherwise an error
/// is returned.
pub fn header<T>(req: &Request<T>, header: &str) -> Result<Option<String>> {
    let Some(data) = req.metadata().get(header).map(|data| data.to_str()) else {
        return Ok(None);
    };

    data.map(|data| data.to_string())
        .map(|value| Some(value))
        .map_err(on_error!(Error, "parsing header data to str"))
}
