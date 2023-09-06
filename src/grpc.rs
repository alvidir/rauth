//! gRPC utilities for managing request's headers.

use crate::on_error;
use tonic::Request;

/// Given a gPRC request, returns the value of the provided header's key if any, otherwise an error
/// is returned.
pub fn get_header<T, Err>(req: &Request<T>, header: &str) -> Result<Option<String>, Err>
where
    Err: From<String>,
{
    let Some(data) = req.metadata().get(header).map(|data| data.to_str()) else {
        return Ok(None);
    };

    data.map(|data| data.to_string())
        .map(|value| Some(value))
        .map_err(on_error!("parsing header data to str"))
}
