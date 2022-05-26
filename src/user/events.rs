use super::{application::UserRepository, domain::User};
use futures_lite::stream::StreamExt;
use lapin::{
    options::*, types::FieldTable, BasicProperties, Channel, Connection, ConnectionProperties,
    ExchangeKind,
};
use std::error::Error;

pub struct RabbitMqUserChannel<'a> {
    pub channel: &'a Channel,
}

impl<'a> RabbitMqUserChannel<'a> {
    pub fn emit_user_created(user: &mut User) -> Result<(), Box<dyn Error>> {}
}
