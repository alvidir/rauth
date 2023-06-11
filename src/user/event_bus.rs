use super::{application::EventBus, domain::User};
use crate::{
    rabbitmq::EventKind,
    result::{Error, Result},
};
use async_trait::async_trait;
use deadpool_lapin::Pool;
use lapin::{options::*, BasicProperties};
use serde_json;

#[derive(Serialize, Deserialize)]
struct UserEventPayload<'a> {
    pub(super) user_id: i32,
    pub(super) user_name: &'a str,
    pub(super) user_email: &'a str,
    pub(super) event_issuer: &'a str,
    pub(super) event_kind: EventKind,
}

pub struct RabbitMqUserBus<'a> {
    pub pool: &'a Pool,
    pub exchange: &'a str,
    pub issuer: &'a str,
}

#[async_trait]
impl<'a> EventBus for RabbitMqUserBus<'a> {
    async fn emit_user_created(&self, user: &User) -> Result<()> {
        let event = UserEventPayload {
            user_id: user.get_id(),
            user_name: user.get_name().split('@').collect::<Vec<&str>>()[0],
            user_email: user.get_email(),
            event_issuer: self.issuer,
            event_kind: EventKind::Created,
        };

        let payload = serde_json::to_string(&event)
            .map(|str| str.into_bytes())
            .map_err(|err| {
                error!(
                    "{} serializing \"user created\" event data to json: {}",
                    Error::Unknown,
                    err
                );
                Error::Unknown
            })?;

        let connection = self.pool.get().await.map_err(|err| {
            error!(
                "{} pulling connection from rabbitmq pool: {}",
                Error::Unknown,
                err
            );
            Error::Unknown
        })?;

        connection
            .create_channel()
            .await
            .map_err(|err| {
                error!("{} creating rabbitmq channel: {}", Error::Unknown, err);
                Error::Unknown
            })?
            .basic_publish(
                self.exchange,
                "",
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default(),
            )
            .await
            .map_err(|err| {
                error!(
                    "{} emititng \"user created\" event: {}",
                    Error::Unknown,
                    err
                );
                Error::Unknown
            })?
            .await
            .map_err(|err| {
                error!(
                    "{} confirming \"user created\" event reception: {}",
                    Error::Unknown,
                    err
                );
                Error::Unknown
            })?;

        Ok(())
    }
}
