#[macro_use]
extern crate tracing;
#[macro_use]
extern crate serde;

pub mod cache;
pub mod config;
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "rabbitmq")]
pub mod rabbitmq;
#[cfg(feature = "redis-cache")]
pub mod redis;
pub mod secret;
// pub mod session;
#[cfg(feature = "smtp")]
pub mod smtp;
pub mod token;
#[cfg(feature = "tracer")]
pub mod tracer;
pub mod user;

mod crypto;
#[cfg(feature = "grpc")]
mod grpc;
#[cfg(feature = "rest")]
mod http;

macro_rules! on_error {
    ($type:ty, $msg:tt) => {
        |error| -> $type {
            error!(error = error.to_string(), $msg,);
            <$type>::from(error)
        }
    };

    ($msg:tt) => {
        |error| {
            error!(error = error.to_string(), $msg,);
            error.into()
        }
    };
}

pub(crate) use on_error;
