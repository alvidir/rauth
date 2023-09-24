use serde::{Deserialize, Serialize};
use std::{fmt::Debug, hash::Hash};

/// Represents all the possible kind of events that may be handled or emited.
#[derive(Debug, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Created,
    Deleted,
}
