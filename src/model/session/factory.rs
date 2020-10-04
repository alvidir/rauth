use crate::model::session::{session::SessionImplementation,
                            traits::Session};

pub fn new_session() -> Box<dyn Session> {
    Box::new(SessionImplementation{id: "hello world", deadline: 32})
}