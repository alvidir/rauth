use super::{User, UserID};
use crate::event::domain::{Event, EventKind};
use crate::user::error::{Error, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserEventPayload<'a> {
    pub(super) user_id: UserID,
    pub(super) user_name: &'a str,
    pub(super) user_email: &'a str,
    pub(super) event_kind: EventKind,
}

impl<'a> TryFrom<UserEventPayload<'a>> for Event {
    type Error = Error;

    fn try_from(value: UserEventPayload) -> Result<Self> {
        Event::try_from(value).map_err(Into::into)
    }
}

impl<'a> UserEventPayload<'a> {
    pub fn new(kind: EventKind, user: &'a User) -> Self {
        Self {
            user_id: user.id,
            user_name: user.credentials.email.username(),
            user_email: user.credentials.email.as_ref(),
            event_kind: kind,
        }
    }
}
