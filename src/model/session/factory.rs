use std::time::Duration;
use crate::model::client::traits::Client;
use crate::model::session::{session::SessionImplementation,
                            traits::Session};

pub fn new_session(client: &'static (dyn Client + 'static), timeout: Duration) -> Box<dyn Session> {
    Box::new(SessionImplementation::new(client, timeout))
}

// pub fn get_session(id: &'static str) -> Box<dyn Session> {
//     // TODO: find session & build
// }