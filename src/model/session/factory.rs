use crate::model::client::traits::Client;
use crate::model::session::{session::SessionImplementation,
                            traits::Session};

pub fn new_session(client: &'static (dyn Client + 'static)) -> Box<dyn Session> {
    Box::new(SessionImplementation::new(client))
}