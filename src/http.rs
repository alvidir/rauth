use crate::base64;
use crate::result::Error;
use actix_web::HttpRequest;

fn get_header(req: HttpRequest, header: &str) -> Result<String, String> {
    req.headers()
        .get(header)
        .ok_or(Error::NotFound.to_string())
        .and_then(|header| header.to_str().map_err(|err| err.to_string()))
        .map(Into::into)
}

/// Given an http request, returns the base64 decoded value of the provided header's key if any, otherwise
/// an error is returned.
pub fn get_encoded_header(req: HttpRequest, header: &str) -> Result<String, String> {
    let header = get_header(req, header)?;
    base64::decode_str(&header)
}
