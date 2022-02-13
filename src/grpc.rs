use tonic::Request;
use tonic::Status;
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