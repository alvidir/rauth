use super::{User, UserID};
use crate::event::domain::EventKind;

#[derive(Debug, Hash, Serialize)]
pub struct UserEventPayload<'a> {
    pub(super) user_id: UserID,
    pub(super) user_name: &'a str,
    pub(super) user_email: &'a str,
    pub(super) event_kind: EventKind,
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
