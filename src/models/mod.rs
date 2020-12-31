pub mod session;
pub mod client;
pub mod provider;

pub trait Gateway {
    fn insert(&self) -> Result<(), String>;
    fn update(&self) -> Result<(), String>;
    fn delete(&self) -> Result<(), String>;
}