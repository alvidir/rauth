use crate::engines;
use base64::Engine;
use tonic::{Request, Status};

use crate::errors;

pub fn get_header<T>(req: &Request<T>, header: &str) -> Result<String, Status> {
    let data = req
        .metadata()
        .get(header)
        .ok_or_else(|| Status::aborted(errors::ERR_NOT_FOUND))
        .map(|data| data.to_str())?;

    data.map(|data| data.to_string()).map_err(|err| {
        warn!(
            "{} parsing header data to str: {}",
            errors::ERR_INVALID_HEADER,
            err
        );
        Status::aborted(errors::ERR_INVALID_HEADER)
    })
}

pub fn get_encoded_header<T>(request: &Request<T>, header: &str) -> Result<String, Status> {
    let header = get_header(request, header)?;
    let header = engines::B64.decode(header).map_err(|err| {
        warn!(
            "{} decoding header from base64: {}",
            errors::ERR_INVALID_HEADER,
            err
        );
        Status::unknown(errors::ERR_INVALID_HEADER)
    })?;

    let header = String::from_utf8(header).map_err(|err| {
        warn!(
            "{} parsing header to str: {}",
            errors::ERR_INVALID_HEADER,
            err
        );
        Status::unknown(errors::ERR_INVALID_HEADER)
    })?;

    Ok(header)
}
