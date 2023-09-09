use crate::on_error;
use actix_web::HttpRequest;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Header(#[from] actix_web::http::header::ToStrError),
}

pub fn header(req: HttpRequest, header: &str) -> Result<Option<String>> {
    let Some(header) = req.headers().get(header) else {
        return Ok(None);
    };

    header
        .to_str()
        .map(|value| Some(value.to_string()))
        .map_err(on_error!(Error, "parsing header data to str"))
}
