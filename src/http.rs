use crate::result::{Error, Result};
use actix_web::{HttpRequest, HttpResponse};

impl From<Error> for HttpResponse {
    fn from(value: Error) -> Self {
        match value {
            Error::Unknown => HttpResponse::InternalServerError().body(value.to_string()),
            Error::NotFound => HttpResponse::NotFound().finish(),
            Error::NotAvailable => HttpResponse::ServiceUnavailable().finish(),
            Error::Unauthorized => HttpResponse::Unauthorized().finish(),
            Error::InvalidToken | Error::InvalidFormat | Error::InvalidHeader => {
                HttpResponse::BadRequest().body(value.to_string())
            }

            Error::WrongCredentials => HttpResponse::Forbidden().finish(),
            Error::RegexNotMatch => HttpResponse::NotAcceptable().finish(),
        }
    }
}

pub fn get_header(req: HttpRequest, header: &str) -> Result<String> {
    req.headers()
        .get(header)
        .ok_or(Error::NotFound)
        .and_then(|header| {
            header.to_str().map_err(|err| {
                warn!(error = err.to_string(), "parsing header data to str",);
                Error::InvalidHeader
            })
        })
        .map(Into::into)
}
