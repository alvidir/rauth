use super::{application::EventBus, domain::User};
use crate::errors;
use async_trait::async_trait;
use lapin::{options::*, BasicProperties, Channel};
use serde_json;
use std::error::Error;

#[derive(Serialize, Deserialize)]
enum EventKind {
    Created,
}

#[derive(Serialize, Deserialize)]
struct UserEvent<'a> {
    pub id: i32,
    pub name: &'a str,
    pub email: &'a str,
    pub kind: EventKind,
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
            name: user.get_name().split('@').collect::<Vec<&str>>()[0],
            email: user.get_email(),
            kind: EventKind::Created,
        };

        let payload = serde_json::to_string(&event)
            .map(|str| str.into_bytes())
            .map_err(|err| {
                error!(
                    "{} serializing \"user created\" event data to json: {}",
                    errors::ERR_UNKNOWN,
                    err
                );
                errors::ERR_UNKNOWN
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
                    errors::ERR_UNKNOWN,
                    err
                );
                errors::ERR_UNKNOWN
            })?
            .await
            .map_err(|err| {
                error!(
                    "{} confirming \"user created\" event reception: {}",
                    errors::ERR_UNKNOWN,
                    err
                );
                errors::ERR_UNKNOWN
            })?;

        Ok(())
    }
}
