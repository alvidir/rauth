use super::error::{Error, Result};
use serde::Serialize;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

/// Represents all the possible kind of events that may be handled or emited.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Created,
    Deleted,
}

/// Represents an event that is about to be published.
#[derive(Debug)]
pub struct Event {
    pub(super) id: String,
    pub(super) payload: String,
}

impl Event {
    pub fn try_from<T>(value: T) -> Result<Self>
    where
        T: Serialize,
    {
        let payload = serde_json::to_string(&value).map_err(Error::from)?;

        let mut hasher = DefaultHasher::new();
        payload.hash(&mut hasher);

        Ok(Self {
            id: hasher.finish().to_string(),
            payload,
        })
    }
}
