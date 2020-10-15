use crate::model::app::{app::AppImplementation,
                        traits::App};

pub fn get_app(name: &'static str) -> Box<dyn App> {
    Box::new(AppImplementation::new(name, "addr"))
}