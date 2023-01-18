//! RabbitMQ utilities for managing events handlering and emitions.

use serde::{Deserialize, Serialize};

/// Represents all the possible kind of events that may be handled or emited.
#[derive(strum_macros::Display, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
pub enum EventKind {
    Created,
    Deleted,
}
