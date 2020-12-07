pub trait Controller {
    fn get_description(&self) -> &str;
    fn get_addr(&self) -> &str;
}