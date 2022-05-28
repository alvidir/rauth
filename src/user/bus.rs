use super::{application::EventBus, domain::User};
use crate::constants;
use async_trait::async_trait;
use lapin::{options::*, BasicProperties, Channel};
use serde_json;
use std::error::Error;

#[derive(Serialize, Deserialize)]
enum EventKind {
    CREATED = 0,
}

#[derive(Serialize, Deserialize)]
struct UserEvent<'a> {
    pub id: i32,
    pub name: &'a str,
    pub email: &'a str,
    pub kind: EventKind
}

pub struct RabbitMqUserBus<'a> {
    pub channel: &'a Channel,
    pub bus: &'a str,
}

#[async_trait]
impl<'a> EventBus for RabbitMqUserBus<'a> {
    async fn emit_user_created(&self, user: &User) -> Result<(), Box<dyn Error>> {
        let event = UserEvent {
            id: user.get_id(),
            name: user.get_name(),
            email: user.get_email(),
            kind: EventKind::CREATED
        };

        let payload = serde_json::to_string(&event)
            .map(|str| str.into_bytes())
            .map_err(|err| {
                error!(
                    "{} serializing \"user created\" event data to json: {}",
                    constants::ERR_UNKNOWN,
                    err
                );
                constants::ERR_UNKNOWN
            })?;

        self.channel
            .basic_publish(
                self.bus,
                "",
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default(),
            )
            .await
            .map_err(|err| {
                error!(
                    "{} emititng \"user created\" event: {}",
                    constants::ERR_UNKNOWN,
                    err
                );
                constants::ERR_UNKNOWN
            })?
            .await
            .map_err(|err| {
                error!(
                    "{} confirming \"user created\" event reception: {}",
                    constants::ERR_UNKNOWN,
                    err
                );
                constants::ERR_UNKNOWN
            })?;

        Ok(())
    }
}
