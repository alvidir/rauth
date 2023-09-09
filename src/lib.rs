#[macro_use]
extern crate tracing;
#[macro_use]
extern crate serde;

pub mod cache;
pub mod config;
pub mod mfa;
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "rabbitmq")]
pub mod rabbitmq;
#[cfg(feature = "redis-cache")]
pub mod redis;
pub mod secret;
pub mod session;
#[cfg(feature = "smtp")]
pub mod smtp;
pub mod token;
#[cfg(feature = "tracer")]
pub mod tracer;
pub mod user;

mod base64;
#[cfg(feature = "grpc")]
mod grpc;
#[cfg(feature = "rest")]
mod http;

macro_rules! on_error {
    ($from:ty as $to:ty, $msg:tt) => {
        |error: $from| -> $to {
            error!(error = error.to_string(), $msg,);
            error.into()
        }
    };
    ($to:ty, $msg:tt) => {
        |error| -> $to {
            error!(error = error.to_string(), $msg,);
            <$to>::from(error)
        }
    };
}

pub(crate) use on_error;
