use crate::model::client::{client::ClientImplementation,
                            traits::Client};

pub fn new_client() -> Box<dyn Client> {
    Box::new(ClientImplementation::new())
}