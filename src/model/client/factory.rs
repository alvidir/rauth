use crate::model::client::{client::ClientImplementation,
                        traits::Client};

pub fn get_client(id: &'static str) -> Box<dyn Client> {
    Box::new(ClientImplementation::new())
}