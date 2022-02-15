use tonic::{
    Request, Status,
};

use crate::constants;

pub fn get_header<T>(req: &Request<T>, header: &str) -> Result<String, Status> {
    let data = req.metadata().get(header)
        .ok_or(Status::aborted(constants::ERR_NOT_AVAILABLE))
        .map(|data| data.to_str())?;

    data.map(|data| data.to_string())
        .map_err(|err| {
            warn!("{} parsing header data to str: {}", constants::ERR_INVALID_TOKEN, err);
            Status::aborted(constants::ERR_INVALID_TOKEN)
        })
}

pub fn get_encoded_header<T>(request: &Request<T>, header: &str) -> Result<String, Status> {
    let header = get_header(&request, header)?;
    let header = base64::decode(header)
        .map_err(|err| {
            warn!("{} decoding header from base64: {}", constants::ERR_INVALID_TOKEN, err);
            Status::unknown(constants::ERR_INVALID_TOKEN)
        })?;

    let header = String::from_utf8(header)
        .map_err(|err| {
            warn!("{} parsing header to str: {}", constants::ERR_INVALID_TOKEN, err);
            Status::unknown(constants::ERR_INVALID_TOKEN)
        })?;

    Ok(header)
}