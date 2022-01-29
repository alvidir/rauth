use tonic::Request;
use tonic::Status;
use crate::constants;

pub fn get_header<T>(req: &Request<T>, header: &str) -> Result<String, Status> {
    match req.metadata().get(header) {
        Some(data) => data.to_str()
            .map(|jwt| jwt.to_string())
            .map_err(|err| {
                error!("{}: {}", constants::ERR_PARSE_HEADER, err);
                Status::aborted(constants::ERR_PARSE_HEADER)
            }),

        None => {
            warn!("{}", constants::ERR_NOT_FOUND);
            return Err(Status::unauthenticated(constants::ERR_NOT_FOUND));
        }
    }    
}