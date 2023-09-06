use crate::on_error;
use actix_web::{HttpRequest, HttpResponse};

// impl From<Error> for HttpResponse {
//     fn from(value: Error) -> Self {
//         match value {
//             Error::Unknown => HttpResponse::InternalServerError().body(value.to_string()),
//             Error::NotFound => HttpResponse::NotFound().finish(),
//             Error::NotAvailable => HttpResponse::ServiceUnavailable().finish(),
//             Error::Unauthorized => HttpResponse::Unauthorized().finish(),
//             Error::InvalidToken | Error::InvalidFormat | Error::InvalidHeader => {
//                 HttpResponse::BadRequest().body(value.to_string())
//             }

//             Error::WrongCredentials => HttpResponse::Forbidden().finish(),
//             Error::RegexNotMatch => HttpResponse::NotAcceptable().finish(),
//         }
//     }
// }

pub fn get_header<Err>(req: HttpRequest, header: &str) -> Result<Option<String>, Err>
where
    Err: From<String>,
{
    let Some(header) = req.headers().get(header) else {
        return Ok(None);
    };

    header
        .to_str()
        .map(|value| Some(value.to_string()))
        .map_err(on_error!("parsing header data to str"))
}
