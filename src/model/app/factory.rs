use crate::model::app::{app::AppImplementation,
                        traits::App};

pub fn new_app(name: &'static str, addr: &'static str) -> Box<dyn App> {
    Box::new(AppImplementation::new(name, addr))
}